use bencher_rbac::{
    user::{OrganizationRoles, ProjectRoles},
    Organization, Project, Server, User as RbacUser,
};

use bencher_json::Jwt;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use dropshot::{HttpError, RequestContext};
use http::StatusCode;
use oso::{PolarValue, ToPolar};

use crate::{
    context::{ApiContext, DbConnection, Rbac},
    model::{organization::OrganizationId, project::ProjectId},
    schema,
};

use super::UserId;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: UserId,
    pub organizations: Vec<OrganizationId>,
    pub projects: Vec<OrgProjectId>,
    pub rbac: RbacUser,
}

impl From<OrganizationId> for Organization {
    fn from(org_id: OrganizationId) -> Self {
        Self {
            id: org_id.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OrgProjectId {
    pub organization_id: OrganizationId,
    pub id: ProjectId,
}

impl From<OrgProjectId> for Project {
    fn from(proj_id: OrgProjectId) -> Self {
        Self {
            organization_id: proj_id.organization_id.to_string(),
            id: proj_id.id.to_string(),
        }
    }
}

impl AuthUser {
    pub async fn new(rqctx: &RequestContext<ApiContext>) -> Result<Self, HttpError> {
        let request = &rqctx.request;

        const EXPECTED: &str = "Expected format is `Authorization: Bearer <bencher.api.token>`.\nWhere `<bencher.api.token>` is your Bencher API token.";
        let headers = request
            .headers()
            .get("Authorization")
            .ok_or_else(|| {
                HttpError::for_client_error(
                    None,
                    StatusCode::UNAUTHORIZED,
                    format!("Request is missing \"Authorization\" header.\n{EXPECTED}"),
                )
            })?
            .to_str()
            .map_err(|e| {
                HttpError::for_client_error(
                    None,
                    StatusCode::UNAUTHORIZED,
                    format!("Request has an invalid \"Authorization\" header: {e}\n{EXPECTED}"),
                )
            })?;
        let (_, token) = headers.split_once("Bearer ").ok_or_else(|| {
            HttpError::for_client_error(
                None,
                StatusCode::UNAUTHORIZED,
                format!("Request is missing \"Authorization\" Bearer.\n{EXPECTED}"),
            )
        })?;
        let token = token.trim();
        let jwt: Jwt = token.parse().map_err(|e| {
            HttpError::for_client_error(
                None,
                StatusCode::UNAUTHORIZED,
                format!("Malformed JSON Web Token: {e}"),
            )
        })?;

        let context = rqctx.context();
        let conn = &mut *context.conn().await;
        let claims = context.secret_key.validate_client(&jwt).map_err(|e| {
            HttpError::for_client_error(
                None,
                StatusCode::UNAUTHORIZED,
                format!("Failed to validate JSON Web Token: {e}"),
            )
        })?;

        let email = claims.email();
        let (user_id, admin, locked) = schema::user::table
            .filter(schema::user::email.eq(email))
            .select((schema::user::id, schema::user::admin, schema::user::locked))
            .first::<(UserId, bool, bool)>(conn)
            .map_err(|e| {
                HttpError::for_client_error(
                    None,
                    StatusCode::NOT_FOUND,
                    format!("Failed to find user ({email}): {e}"),
                )
            })?;

        if locked {
            return Err(HttpError::for_client_error(
                None,
                StatusCode::UNAUTHORIZED,
                format!("User account is locked ({email})"),
            ));
        }

        let (org_ids, org_roles) = Self::organization_roles(conn, user_id, email)?;
        let (proj_ids, proj_roles) = Self::project_roles(conn, user_id, email)?;
        let rbac = RbacUser {
            admin,
            locked,
            organizations: org_roles,
            projects: proj_roles,
        };

        Ok(Self {
            id: user_id,
            organizations: org_ids,
            projects: proj_ids,
            rbac,
        })
    }

    fn organization_roles(
        conn: &mut DbConnection,
        user_id: UserId,
        email: &str,
    ) -> Result<(Vec<OrganizationId>, OrganizationRoles), HttpError> {
        let roles = schema::organization_role::table
            .filter(schema::organization_role::user_id.eq(user_id))
            .order(schema::organization_role::organization_id)
            .select((
                schema::organization_role::organization_id,
                schema::organization_role::role,
            ))
            .load::<(OrganizationId, String)>(conn)
            .map_err(|e| {
                crate::error::issue_error(
                    StatusCode::NOT_FOUND,
                    "User can't query organization roles",
                    &format!("My user ({email}) on Bencher failed to query organization roles."),
                    e,
                )
            })?;

        let org_ids = roles.iter().map(|(org_id, _)| *org_id).collect();
        let roles = roles
            .into_iter()
            .filter_map(|(org_id, role)| match role.parse() {
                Ok(role) => Some((org_id.to_string(), role)),
                Err(e) => {
                    let _ = crate::error::issue_error(
                        StatusCode::NOT_FOUND,
                        "Failed to parse organization role",
                        &format!("My user ({email}) on Bencher has an invalid organization role ({role})."),
                        e,
                    );
                    None
                },
            })
            .collect();

        Ok((org_ids, roles))
    }

    fn project_roles(
        conn: &mut DbConnection,
        user_id: UserId,
        email: &str,
    ) -> Result<(Vec<OrgProjectId>, ProjectRoles), HttpError> {
        let roles = schema::project_role::table
            .filter(schema::project_role::user_id.eq(user_id))
            .inner_join(
                schema::project::table.on(schema::project_role::project_id.eq(schema::project::id)),
            )
            .order(schema::project_role::project_id)
            .select((
                schema::project::organization_id,
                schema::project_role::project_id,
                schema::project_role::role,
            ))
            .load::<(OrganizationId, ProjectId, String)>(conn)
            .map_err(|e| {
                crate::error::issue_error(
                    StatusCode::NOT_FOUND,
                    "User can't query project roles",
                    &format!("My user ({email}) on Bencher failed to query project roles."),
                    e,
                )
            })?;

        let ids = roles
            .iter()
            .map(|(org_id, id, _)| OrgProjectId {
                organization_id: *org_id,
                id: *id,
            })
            .collect();
        let roles = roles
            .into_iter()
            .filter_map(|(_, id, role)| match role.parse() {
                Ok(role) => Some((id.to_string(), role)),
                Err(e) => {
                    let _ = crate::error::issue_error(
                        StatusCode::NOT_FOUND,
                        "Failed to parse project role",
                        &format!(
                            "My user ({email}) on Bencher has an invalid project role ({role})."
                        ),
                        e,
                    );
                    None
                },
            })
            .collect();

        Ok((ids, roles))
    }

    pub fn is_admin(&self, rbac: &Rbac) -> bool {
        rbac.is_allowed_unwrap(
            self,
            bencher_rbac::server::Permission::Administer,
            Server {},
        )
    }

    pub fn organizations(
        &self,
        rbac: &Rbac,
        action: bencher_rbac::organization::Permission,
    ) -> Vec<OrganizationId> {
        self.organizations
            .iter()
            .filter_map(|org_id| {
                rbac.is_allowed_unwrap(self, action, Organization::from(*org_id))
                    .then_some(*org_id)
            })
            .collect()
    }

    pub fn projects(
        &self,
        rbac: &Rbac,
        action: bencher_rbac::project::Permission,
    ) -> Vec<ProjectId> {
        self.projects
            .iter()
            .filter_map(|proj_id| {
                rbac.is_allowed_unwrap(self, action, Project::from(*proj_id))
                    .then_some(proj_id.id)
            })
            .collect()
    }
}

impl ToPolar for &AuthUser {
    fn to_polar(self) -> PolarValue {
        self.rbac.clone().to_polar()
    }
}
