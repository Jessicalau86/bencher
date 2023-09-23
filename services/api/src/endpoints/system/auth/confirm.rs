use bencher_json::{system::auth::JsonAuthUser, JsonAuthToken};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use dropshot::{endpoint, HttpError, RequestContext, TypedBody};

use crate::{
    context::ApiContext,
    endpoints::{
        endpoint::{pub_response_accepted, ResponseAccepted},
        Endpoint, Method,
    },
    model::user::QueryUser,
    schema,
    util::cors::{get_cors, CorsResponse},
    ApiError,
};

use super::{Resource, CLIENT_TOKEN_TTL};

const CONFIRM_RESOURCE: Resource = Resource::Confirm;

#[allow(clippy::unused_async)]
#[endpoint {
    method = OPTIONS,
    path =  "/v0/auth/confirm",
    tags = ["auth"]
}]
pub async fn auth_confirm_options(
    _rqctx: RequestContext<ApiContext>,
) -> Result<CorsResponse, HttpError> {
    Ok(get_cors::<ApiContext>())
}

#[endpoint {
    method = POST,
    path = "/v0/auth/confirm",
    tags = ["auth"]
}]
pub async fn auth_confirm_post(
    rqctx: RequestContext<ApiContext>,
    body: TypedBody<JsonAuthToken>,
) -> Result<ResponseAccepted<JsonAuthUser>, HttpError> {
    let endpoint = Endpoint::new(CONFIRM_RESOURCE, Method::Post);

    let json = post_inner(rqctx.context(), body.into_inner())
        .await
        .map_err(|e| {
            if let ApiError::HttpError(e) = e {
                e
            } else {
                endpoint.err(e).into()
            }
        })?;

    pub_response_accepted!(endpoint, json)
}

async fn post_inner(
    context: &ApiContext,
    json_token: JsonAuthToken,
) -> Result<JsonAuthUser, ApiError> {
    let conn = &mut *context.conn().await;

    let claims = context
        .secret_key
        .validate_auth(&json_token.token)
        .map_err(ApiError::from)?;

    let email = claims.email();
    let user = schema::user::table
        .filter(schema::user::email.eq(email))
        .first::<QueryUser>(conn)
        .map_err(ApiError::from)?
        .into_json()?;

    let token = context
        .secret_key
        .new_client(email.parse()?, CLIENT_TOKEN_TTL)
        .map_err(ApiError::from)?;

    Ok(JsonAuthUser { user, token })
}
