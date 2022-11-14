use crate::{
    auth::{authorize, Role},
    gql::schema::Context as ResourcesContext,
};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Method,
};
use std::{convert::Infallible, sync::Arc};
use warp::{filters::cors::Builder, header::headers_cloned};
use warp::{Filter, Rejection};

pub fn with_cors() -> Builder {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "Sec-Fetch-Mode",
            "Sec-Fetch-Dest",
            "Sec-Fetch-Site",
            "Mode",
            "Credentials",
            reqwest::header::ACCEPT.as_str(),
            reqwest::header::ACCEPT_CHARSET.as_str(),
            reqwest::header::ACCEPT_ENCODING.as_str(),
            reqwest::header::ACCEPT_LANGUAGE.as_str(),
            reqwest::header::ACCEPT_RANGES.as_str(),
            reqwest::header::USER_AGENT.as_str(),
            reqwest::header::REFERER.as_str(),
            reqwest::header::REFERRER_POLICY.as_str(),
            reqwest::header::ORIGIN.as_str(),
            reqwest::header::ALLOW.as_str(),
            reqwest::header::COOKIE.as_str(),
            reqwest::header::HOST.as_str(),
            reqwest::header::ACCESS_CONTROL_REQUEST_METHOD.as_str(),
            reqwest::header::ACCESS_CONTROL_REQUEST_HEADERS.as_str(),
            reqwest::header::ACCESS_CONTROL_EXPOSE_HEADERS.as_str(),
            reqwest::header::ACCESS_CONTROL_MAX_AGE.as_str(),
            reqwest::header::ACCESS_CONTROL_ALLOW_METHODS.as_str(),
            reqwest::header::ACCESS_CONTROL_ALLOW_CREDENTIALS.as_str(),
            reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN.as_str(),
            reqwest::header::ACCESS_CONTROL_ALLOW_HEADERS.as_str(),
            reqwest::header::CONTENT_TYPE.as_str(),
            reqwest::header::AUTHORIZATION.as_str(),
            reqwest::header::UPGRADE.as_str(),
            reqwest::header::UPGRADE_INSECURE_REQUESTS.as_str(),
        ])
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::DELETE,
            Method::OPTIONS,
            Method::PUT,
        ]);

    cors
}

pub fn with_resources_context(
    resources_ctx: Arc<ResourcesContext>,
) -> impl warp::Filter<Extract = (Arc<ResourcesContext>,), Error = Infallible> + Clone {
    warp::any().map(move || Arc::clone(&resources_ctx))
}

pub fn with_auth(
    roles: Vec<Role>,
) -> impl Filter<Extract = (uuid::Uuid,), Error = Rejection> + Clone {
    headers_cloned()
        .map(move |headers: HeaderMap<HeaderValue>| (roles.clone(), headers))
        .and_then(authorize)
}
