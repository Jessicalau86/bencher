use std::collections::HashMap;

use bencher_json::{
    JsonDirection, JsonEmpty, JsonNewReport, JsonPagination, JsonReport, JsonReports, ResourceId,
};
use bencher_rbac::project::Permission;
use diesel::{dsl::count, BelongingToDsl, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use dropshot::{endpoint, HttpError, Path, Query, RequestContext, TypedBody};
use schemars::JsonSchema;
use serde::Deserialize;
use slog::Logger;
use uuid::Uuid;

use crate::{
    context::ApiContext,
    endpoints::{
        endpoint::{pub_response_ok, response_accepted, response_ok, ResponseAccepted, ResponseOk},
        Endpoint, Method,
    },
    model::project::{
        branch::QueryBranch,
        report::{results::ReportResults, InsertReport, QueryReport},
        testbed::QueryTestbed,
        version::{InsertVersion, QueryVersion, VersionId},
        QueryProject,
    },
    model::user::auth::AuthUser,
    schema,
    util::{
        cors::{get_cors, CorsResponse},
        error::database_map,
    },
    ApiError,
};

use super::Resource;

const REPORT_RESOURCE: Resource = Resource::Report;

#[derive(Deserialize, JsonSchema)]
pub struct ProjReportsParams {
    pub project: ResourceId,
}

pub type ProjReportsPagination = JsonPagination<ProjReportsSort>;

#[derive(Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProjReportsSort {
    #[default]
    DateTime,
}

#[allow(clippy::unused_async)]
#[endpoint {
    method = OPTIONS,
    path =  "/v0/projects/{project}/reports",
    tags = ["projects", "reports"]
}]
pub async fn proj_reports_options(
    _rqctx: RequestContext<ApiContext>,
    _path_params: Path<ProjReportsParams>,
    _pagination_params: Query<ProjReportsPagination>,
) -> Result<CorsResponse, HttpError> {
    Ok(get_cors::<ApiContext>())
}

#[endpoint {
    method = GET,
    path =  "/v0/projects/{project}/reports",
    tags = ["projects", "reports"]
}]
pub async fn proj_reports_get(
    rqctx: RequestContext<ApiContext>,
    path_params: Path<ProjReportsParams>,
    pagination_params: Query<ProjReportsPagination>,
) -> Result<ResponseOk<JsonReports>, HttpError> {
    let auth_user = AuthUser::new(&rqctx).await.ok();
    let endpoint = Endpoint::new(REPORT_RESOURCE, Method::GetLs);

    let json = get_ls_inner(
        &rqctx.log,
        rqctx.context(),
        auth_user.as_ref(),
        path_params.into_inner(),
        pagination_params.into_inner(),
        endpoint,
    )
    .await
    .map_err(|e| {
        if let ApiError::HttpError(e) = e {
            e
        } else {
            endpoint.err(e).into()
        }
    })?;

    if auth_user.is_some() {
        response_ok!(endpoint, json)
    } else {
        pub_response_ok!(endpoint, json)
    }
}

async fn get_ls_inner(
    log: &Logger,
    context: &ApiContext,
    auth_user: Option<&AuthUser>,
    path_params: ProjReportsParams,
    pagination_params: ProjReportsPagination,
    endpoint: Endpoint,
) -> Result<JsonReports, ApiError> {
    let conn = &mut *context.conn().await;

    let query_project =
        QueryProject::is_allowed_public(conn, &context.rbac, &path_params.project, auth_user)?;

    let mut query = QueryReport::belonging_to(&query_project).into_boxed();

    query = match pagination_params.order() {
        ProjReportsSort::DateTime => match pagination_params.direction {
            Some(JsonDirection::Asc) => query.order((
                schema::report::start_time.asc(),
                schema::report::end_time.asc(),
                schema::report::created.asc(),
            )),
            Some(JsonDirection::Desc) | None => query.order((
                schema::report::start_time.desc(),
                schema::report::end_time.desc(),
                schema::report::created.desc(),
            )),
        },
    };

    Ok(query
        .offset(pagination_params.offset())
        .limit(pagination_params.limit())
        .load::<QueryReport>(conn)
        .map_err(ApiError::from)?
        .into_iter()
        .filter_map(|query| database_map(endpoint, query.into_json(log, conn)))
        .collect())
}

#[endpoint {
    method = POST,
    path =  "/v0/projects/{project}/reports",
    tags = ["projects", "reports"]
}]
// For simplicity, this query makes the assumption that all posts are perfectly
// chronological. That is, a report will never be posted for X after Y has
// already been submitted when X really happened before Y. For implementing git
// bisect more complex logic will be required.
pub async fn proj_report_post(
    rqctx: RequestContext<ApiContext>,
    path_params: Path<ProjReportsParams>,
    body: TypedBody<JsonNewReport>,
) -> Result<ResponseAccepted<JsonReport>, HttpError> {
    let auth_user = AuthUser::new(&rqctx).await?;
    let endpoint = Endpoint::new(REPORT_RESOURCE, Method::Post);

    let json = post_inner(
        &rqctx.log,
        rqctx.context(),
        path_params.into_inner(),
        body.into_inner(),
        &auth_user,
    )
    .await
    .map_err(|e| {
        if let ApiError::HttpError(e) = e {
            e
        } else {
            endpoint.err(e).into()
        }
    })?;

    response_accepted!(endpoint, json)
}

async fn post_inner(
    log: &Logger,
    context: &ApiContext,
    path_params: ProjReportsParams,
    mut json_report: JsonNewReport,
    auth_user: &AuthUser,
) -> Result<JsonReport, ApiError> {
    let conn = &mut *context.conn().await;

    // Verify that the user is allowed
    let project_id = QueryProject::is_allowed(
        conn,
        &context.rbac,
        &path_params.project,
        auth_user,
        Permission::Create,
    )?
    .id;

    // Verify that the branch and testbed are part of the same project
    let branch_id = QueryBranch::from_resource_id(conn, project_id, &json_report.branch)?.id;
    let testbed_id = QueryTestbed::from_resource_id(conn, project_id, &json_report.testbed)?.id;

    // Check to see if the project is public or private
    // If private, then validate that there is an active subscription or license
    #[cfg(feature = "plus")]
    let plan_kind =
        plan_kind::PlanKind::new(conn, context.biller.as_ref(), &context.licensor, project_id)
            .await?;

    // If there is a hash then try to see if there is already a code version for
    // this branch with that particular hash.
    // Otherwise, create a new code version for this branch with/without the hash.
    let version_id = if let Some(hash) = &json_report.hash {
        if let Ok(version_id) = schema::version::table
            .left_join(
                schema::branch_version::table
                    .on(schema::version::id.eq(schema::branch_version::version_id)),
            )
            .filter(schema::branch_version::branch_id.eq(branch_id))
            .filter(schema::version::hash.eq(hash.as_ref()))
            .order(schema::version::number.desc())
            .select(schema::version::id)
            .first::<VersionId>(conn)
        {
            version_id
        } else {
            InsertVersion::increment(conn, project_id, branch_id, Some(hash.clone()))?
        }
    } else {
        InsertVersion::increment(conn, project_id, branch_id, None)?
    };

    let json_settings = json_report.settings.take().unwrap_or_default();
    let adapter = json_settings.adapter.unwrap_or_default();

    // Create a new report and add it to the database
    let insert_report = InsertReport::from_json(
        auth_user.id,
        project_id,
        branch_id,
        version_id,
        testbed_id,
        &json_report,
        adapter,
    );

    diesel::insert_into(schema::report::table)
        .values(&insert_report)
        .execute(conn)
        .map_err(ApiError::from)?;

    let query_report = schema::report::table
        .filter(schema::report::uuid.eq(&insert_report.uuid))
        .first::<QueryReport>(conn)
        .map_err(ApiError::from)?;

    #[cfg(feature = "plus")]
    let mut usage = 0;

    // Process and record the report results
    let mut report_results = ReportResults::new(project_id, branch_id, testbed_id, query_report.id);
    let processed_report = report_results.process(
        log,
        conn,
        json_report
            .results
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<&str>>()
            .as_ref(),
        adapter,
        json_settings,
        #[cfg(feature = "plus")]
        &mut usage,
    );

    #[cfg(feature = "plus")]
    plan_kind
        .check_usage(context.biller.as_ref(), project_id, usage)
        .await?;

    // Don't return the error from processing the report
    // until after the metrics usage has been checked
    processed_report?;

    query_report.into_json(log, conn)
}

#[cfg(feature = "plus")]
mod plan_kind {
    use bencher_billing::{Biller, SubscriptionId};
    use bencher_license::Licensor;

    use crate::{
        context::DbConnection,
        model::project::{ProjectId, QueryProject},
        ApiError,
    };

    pub enum PlanKind {
        Metered(SubscriptionId),
        Licensed { entitlement: u64, prior_usage: u64 },
        None,
    }

    impl PlanKind {
        pub async fn new(
            conn: &mut DbConnection,
            biller: Option<&Biller>,
            licensor: &Licensor,
            project_id: ProjectId,
        ) -> Result<Self, ApiError> {
            if let Some(subscription) = QueryProject::get_subscription(conn, project_id)? {
                if let Some(biller) = biller {
                    let plan_status = biller.get_plan_status(&subscription).await?;
                    if plan_status.is_active() {
                        Ok(PlanKind::Metered(subscription))
                    } else {
                        Err(ApiError::InactivePlanProject(project_id))
                    }
                } else {
                    Err(ApiError::NoBillerProject(project_id))
                }
            } else if let Some((uuid, license)) = QueryProject::get_license(conn, project_id)? {
                let _token_data = licensor.validate_organization(&license, uuid)?;
                // TODO check license entitlements and usage so far
                Ok(PlanKind::Licensed {
                    entitlement: u64::MAX,
                    prior_usage: 0,
                })
            } else if QueryProject::is_public(conn, project_id)? {
                Ok(Self::None)
            } else {
                Err(ApiError::NoPlanProject(project_id))
            }
        }

        pub async fn check_usage(
            &self,
            biller: Option<&Biller>,
            project_id: ProjectId,
            usage: u64,
        ) -> Result<(), ApiError> {
            match self {
                Self::Metered(subscription) => {
                    let Some(biller) = biller else {
                        return Err(ApiError::NoBillerProject(project_id));
                    };
                    biller.record_usage(subscription, usage).await?;
                },
                Self::Licensed {
                    entitlement,
                    prior_usage,
                } =>
                {
                    #[allow(clippy::todo)]
                    if *prior_usage + usage > *entitlement {
                        todo!("Manage license entitlements");
                    }
                },
                Self::None => {},
            }

            Ok(())
        }
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct ProjReportParams {
    pub project: ResourceId,
    pub report: Uuid,
}

#[allow(clippy::unused_async)]
#[endpoint {
    method = OPTIONS,
    path =  "/v0/projects/{project}/reports/{report}",
    tags = ["projects", "reports"]
}]
pub async fn proj_report_options(
    _rqctx: RequestContext<ApiContext>,
    _path_params: Path<ProjReportParams>,
) -> Result<CorsResponse, HttpError> {
    Ok(get_cors::<ApiContext>())
}

#[endpoint {
    method = GET,
    path =  "/v0/projects/{project}/reports/{report}",
    tags = ["projects", "reports"]
}]
pub async fn proj_report_get(
    rqctx: RequestContext<ApiContext>,
    path_params: Path<ProjReportParams>,
) -> Result<ResponseOk<JsonReport>, HttpError> {
    let auth_user = AuthUser::new(&rqctx).await.ok();
    let endpoint = Endpoint::new(REPORT_RESOURCE, Method::GetOne);

    let json = get_one_inner(
        &rqctx.log,
        rqctx.context(),
        path_params.into_inner(),
        auth_user.as_ref(),
    )
    .await
    .map_err(|e| {
        if let ApiError::HttpError(e) = e {
            e
        } else {
            endpoint.err(e).into()
        }
    })?;

    if auth_user.is_some() {
        response_ok!(endpoint, json)
    } else {
        pub_response_ok!(endpoint, json)
    }
}

async fn get_one_inner(
    log: &Logger,
    context: &ApiContext,
    path_params: ProjReportParams,
    auth_user: Option<&AuthUser>,
) -> Result<JsonReport, ApiError> {
    let conn = &mut *context.conn().await;

    let query_project =
        QueryProject::is_allowed_public(conn, &context.rbac, &path_params.project, auth_user)?;

    QueryReport::belonging_to(&query_project)
        .filter(schema::report::uuid.eq(path_params.report.to_string()))
        .select((
            schema::report::id,
            schema::report::uuid,
            schema::report::user_id,
            schema::report::project_id,
            schema::report::branch_id,
            schema::report::version_id,
            schema::report::testbed_id,
            schema::report::adapter,
            schema::report::start_time,
            schema::report::end_time,
            schema::report::created,
        ))
        .first::<QueryReport>(conn)
        .map_err(ApiError::from)?
        .into_json(log, conn)
}

#[endpoint {
    method = DELETE,
    path =  "/v0/projects/{project}/reports/{report}",
    tags = ["projects", "reports"]
}]
pub async fn proj_report_delete(
    rqctx: RequestContext<ApiContext>,
    path_params: Path<ProjReportParams>,
) -> Result<ResponseAccepted<JsonEmpty>, HttpError> {
    let auth_user = AuthUser::new(&rqctx).await?;
    let endpoint = Endpoint::new(REPORT_RESOURCE, Method::Delete);

    let json = delete_inner(rqctx.context(), path_params.into_inner(), &auth_user)
        .await
        .map_err(|e| {
            if let ApiError::HttpError(e) = e {
                e
            } else {
                endpoint.err(e).into()
            }
        })?;

    response_accepted!(endpoint, json)
}

async fn delete_inner(
    context: &ApiContext,
    path_params: ProjReportParams,
    auth_user: &AuthUser,
) -> Result<JsonEmpty, ApiError> {
    let conn = &mut *context.conn().await;

    // Verify that the user is allowed
    let query_project = QueryProject::is_allowed(
        conn,
        &context.rbac,
        &path_params.project,
        auth_user,
        Permission::Delete,
    )?;

    let (report_id, version_id) = QueryReport::belonging_to(&query_project)
        .filter(schema::report::uuid.eq(path_params.report.to_string()))
        .select((schema::report::id, schema::report::version_id))
        .first::<(i32, i32)>(conn)
        .map_err(ApiError::from)?;
    diesel::delete(schema::report::table.filter(schema::report::id.eq(report_id)))
        .execute(conn)
        .map_err(ApiError::from)?;

    // If there are no more reports for this version, delete the version
    // This is necessary because multiple reports can use the same version via a git hash
    // This will cascade and delete all branch versions for this version
    // Before doing so, decrement all greater versions
    if schema::report::table
        .filter(schema::report::version_id.eq(version_id))
        .select(count(schema::report::id))
        .first::<i64>(conn)
        .map_err(ApiError::from)?
        == 0
    {
        let query_version = QueryVersion::get(conn, version_id)?;
        // Get all branches that use this version
        let branches = schema::branch::table
            .inner_join(
                schema::branch_version::table
                    .on(schema::branch_version::branch_id.eq(schema::branch::id)),
            )
            .filter(schema::branch_version::version_id.eq(version_id))
            .select(schema::branch::id)
            .load::<i32>(conn)
            .map_err(ApiError::from)?;

        let mut version_map = HashMap::new();
        // Get all versions greater than this one for each of the branches
        for branch_id in branches {
            schema::version::table
                .filter(schema::version::number.gt(query_version.number))
                .inner_join(
                    schema::branch_version::table
                        .on(schema::branch_version::version_id.eq(schema::version::id)),
                )
                .filter(schema::branch_version::branch_id.eq(branch_id))
                .select((schema::version::id, schema::version::number))
                .load::<(i32, i32)>(conn)
                .map_err(ApiError::from)?
                .into_iter()
                .for_each(|(version_id, version_number)| {
                    version_map.insert(version_id, version_number);
                });
        }

        // For each version greater than this one, decrement the version number
        for (version_id, version_number) in version_map {
            if let Err(e) =
                diesel::update(schema::version::table.filter(schema::version::id.eq(version_id)))
                    .set(schema::version::number.eq(version_number - 1))
                    .execute(conn)
            {
                debug_assert!(
                    false,
                    "Failed to decrement version ({version_id}) number ({version_number}): {e}"
                );
                #[cfg(feature = "sentry")]
                sentry::capture_error(&e);
            }
        }

        // Finally delete the dangling version
        diesel::delete(schema::version::table.filter(schema::version::id.eq(version_id)))
            .execute(conn)
            .map_err(ApiError::from)?;
    }

    Ok(JsonEmpty {})
}
