use bencher_json::JsonEmpty;
use dropshot::{endpoint, HttpError, RequestContext};

use crate::{
    context::ApiContext,
    endpoints::{
        endpoint::{CorsResponse, Get, ResponseOk},
        Endpoint,
    },
};

#[allow(clippy::unused_async)]
#[endpoint {
    method = OPTIONS,
    path =  "/",
    tags = ["server"]
}]
pub async fn server_root_options(
    _rqctx: RequestContext<ApiContext>,
) -> Result<CorsResponse, HttpError> {
    Ok(Endpoint::cors(&[Get.into()]))
}

#[allow(clippy::unused_async)]
#[endpoint {
    method = GET,
    path = "/",
    tags = ["server"]
}]
pub async fn server_root_get(
    _rqctx: RequestContext<ApiContext>,
) -> Result<ResponseOk<JsonEmpty>, HttpError> {
    Ok(Get::pub_response_ok(JsonEmpty {}))
}
