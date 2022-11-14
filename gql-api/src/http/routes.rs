use super::handlers::{
    buyer_create_recovery_code as buyer_create_recovery_code_handler,
    buyer_register_phone as buyer_register_phone_handler, buyer_signup as buyer_signup_handler,
    buyer_verify_phone as buyer_verify_phone_handler,
    buyer_verify_recovery_code as buyer_verify_recovery_code_handler,
    check_username as check_username_handler, create_login_code as create_login_code_handler,
    event_ticket_get_verification_code as event_ticket_get_verification_code_handler,
    get_event_from_verification_code as get_event_from_verification_code_handler,
    health as health_handler, signin as signin_handler,
    signin_with_password as signin_with_password_handler,
    verify_login_code as verify_login_code_handler,
};
use crate::{
    auth::Role,
    filters::{with_auth, with_resources_context},
    gql::schema::Context as ResourcesContext,
};
use std::sync::Arc;
use warp::{
    self,
    log::{Info, Log},
    Filter,
};

/// GET /check_username
pub fn check_username_route(
    resources_ctx: Arc<ResourcesContext>,
    logger: Log<impl Fn(Info<'_>) + Copy + Send + 'static>,
) -> impl Filter<Extract = impl warp::Reply + 'static, Error = warp::Rejection> + Clone + 'static {
    let check_username_route = warp::post()
        .and(warp::path!("api" / "v1" / "check_username"))
        .and(with_resources_context(resources_ctx))
        .and(warp::body::aggregate())
        .and_then(check_username_handler)
        .with(logger);

    check_username_route
}

/// POST /buyer/register-phone
pub fn buyer_register_phone_route(
    resources_ctx: Arc<ResourcesContext>,
    logger: Log<impl Fn(Info<'_>) + Copy + Send + 'static>,
) -> impl Filter<Extract = impl warp::Reply + 'static, Error = warp::Rejection> + Clone + 'static {
    let buyer_register_phone_route = warp::post()
        .and(warp::path!("api" / "v1" / String / "phone"))
        .and(with_resources_context(resources_ctx))
        .and(warp::body::aggregate())
        .and_then(buyer_register_phone_handler)
        .with(logger);

    buyer_register_phone_route
}

/// POST /buyer/verify-phone
pub fn buyer_verify_phone_route(
    resources_ctx: Arc<ResourcesContext>,
    logger: Log<impl Fn(Info<'_>) + Copy + Send + 'static>,
) -> impl Filter<Extract = impl warp::Reply + 'static, Error = warp::Rejection> + Clone + 'static {
    let buyer_verify_phone_route = warp::put()
        .and(warp::path!("api" / "v1" / String / "phone"))
        .and(with_resources_context(resources_ctx))
        .and(warp::body::aggregate())
        .and_then(buyer_verify_phone_handler)
        .with(logger);

    buyer_verify_phone_route
}

/// POST /buyer/signup
pub fn buyer_signup_route(
    resources_ctx: Arc<ResourcesContext>,
    logger: Log<impl Fn(Info<'_>) + Copy + Send + 'static>,
) -> impl Filter<Extract = impl warp::Reply + 'static, Error = warp::Rejection> + Clone + 'static {
    let signup_route = warp::post()
        .and(warp::path!("api" / "v1" / String / "signup"))
        .and(with_resources_context(resources_ctx))
        .and(warp::body::aggregate())
        .and_then(buyer_signup_handler)
        .with(logger);

    signup_route
}

/// POST /signin
pub fn signin_route(
    resources_ctx: Arc<ResourcesContext>,
    logger: Log<impl Fn(Info<'_>) + Copy + Send + 'static>,
) -> impl Filter<Extract = impl warp::Reply + 'static, Error = warp::Rejection> + Clone + 'static {
    let signin_route = warp::post()
        .and(warp::path!("api" / "v1" / String / "signin"))
        .and(with_resources_context(resources_ctx))
        .and(warp::body::aggregate())
        .and_then(signin_handler)
        .with(logger);

    signin_route
}

/// POST /signin_with_pwd
pub fn signin_with_password_route(
    resources_ctx: Arc<ResourcesContext>,
    logger: Log<impl Fn(Info<'_>) + Copy + Send + 'static>,
) -> impl Filter<Extract = impl warp::Reply + 'static, Error = warp::Rejection> + Clone + 'static {
    let signin_with_pwd_route = warp::post()
        .and(warp::path!("api" / "v1" / String / "signin_with_pwd"))
        .and(with_resources_context(resources_ctx))
        .and(warp::body::aggregate())
        .and_then(signin_with_password_handler)
        .with(logger);

    signin_with_pwd_route
}

/// POST /login
pub fn create_login_code_route(
    resources_ctx: Arc<ResourcesContext>,
    logger: Log<impl Fn(Info<'_>) + Copy + Send + 'static>,
) -> impl Filter<Extract = impl warp::Reply + 'static, Error = warp::Rejection> + Clone + 'static {
    let create_login_code_route = warp::post()
        .and(warp::path!("api" / "v1" / String / "login"))
        .and(with_resources_context(resources_ctx))
        .and(warp::body::aggregate())
        .and_then(create_login_code_handler)
        .with(logger);

    create_login_code_route
}

/// PUT /login
pub fn verify_login_code_route(
    resources_ctx: Arc<ResourcesContext>,
    logger: Log<impl Fn(Info<'_>) + Copy + Send + 'static>,
) -> impl Filter<Extract = impl warp::Reply + 'static, Error = warp::Rejection> + Clone + 'static {
    let verify_login_code_route = warp::put()
        .and(warp::path!("api" / "v1" / String / "login"))
        .and(with_resources_context(resources_ctx))
        .and(warp::body::aggregate())
        .and_then(verify_login_code_handler)
        .with(logger);

    verify_login_code_route
}

/// POST /event_ticket_get_verification_code
pub fn event_ticket_get_verification_code_route(
    resources_ctx: Arc<ResourcesContext>,
    logger: Log<impl Fn(Info<'_>) + Copy + Send + 'static>,
) -> impl Filter<Extract = impl warp::Reply + 'static, Error = warp::Rejection> + Clone + 'static {
    let event_ticket_get_verification_code_route = warp::post()
        .and(warp::path!(
            "api" / "v1" / String / "event_ticket_get_verification_code"
        ))
        .and(with_resources_context(resources_ctx))
        .and(warp::body::aggregate())
        .and(with_auth(vec![
            Role::Admin,
            Role::Buyer,
            Role::Seller,
            Role::SuperAdmin,
        ]))
        .and_then(event_ticket_get_verification_code_handler)
        .with(logger);

    event_ticket_get_verification_code_route
}

/// PUT /get_event_from_verification_code
pub fn get_event_from_verification_code_route(
    resources_ctx: Arc<ResourcesContext>,
    logger: Log<impl Fn(Info<'_>) + Copy + Send + 'static>,
) -> impl Filter<Extract = impl warp::Reply + 'static, Error = warp::Rejection> + Clone + 'static {
    let get_event_from_verification_code_route = warp::put()
        .and(warp::path!(
            "api" / "v1" / String / "get_event_from_verification_code"
        ))
        .and(with_resources_context(resources_ctx))
        .and(warp::body::aggregate())
        .and(with_auth(vec![
            Role::Admin,
            Role::Buyer,
            Role::Seller,
            Role::SuperAdmin,
        ]))
        .and_then(get_event_from_verification_code_handler)
        .with(logger);

    get_event_from_verification_code_route
}

/// POST /buyer/recover
pub fn buyer_create_recovery_code_route(
    resources_ctx: Arc<ResourcesContext>,
    logger: Log<impl Fn(Info<'_>) + Copy + Send + 'static>,
) -> impl Filter<Extract = impl warp::Reply + 'static, Error = warp::Rejection> + Clone + 'static {
    let buyer_create_recovery_code_route = warp::post()
        .and(warp::path!("api" / "v1" / String / "recover"))
        .and(with_resources_context(resources_ctx))
        .and(warp::body::aggregate())
        .and_then(buyer_create_recovery_code_handler)
        .with(logger);

    buyer_create_recovery_code_route
}

/// PUT /buyer/recover
pub fn buyer_verify_recovery_code_route(
    resources_ctx: Arc<ResourcesContext>,
    logger: Log<impl Fn(Info<'_>) + Copy + Send + 'static>,
) -> impl Filter<Extract = impl warp::Reply + 'static, Error = warp::Rejection> + Clone + 'static {
    let buyer_verify_recovery_code_route = warp::put()
        .and(warp::path!("api" / "v1" / String / "recover"))
        .and(with_resources_context(resources_ctx))
        .and(warp::body::aggregate())
        .and_then(buyer_verify_recovery_code_handler)
        .with(logger);

    buyer_verify_recovery_code_route
}

/// GET /health
pub fn healthcheck_route(
    resources_ctx: Arc<ResourcesContext>,
    logger: Log<impl Fn(Info<'_>) + Copy + Send + 'static>,
) -> impl Filter<Extract = impl warp::Reply + 'static, Error = warp::Rejection> + Clone + 'static {
    let healthcheck_route = warp::path!("health")
        .and(with_resources_context(resources_ctx.clone()))
        .and_then(health_handler)
        .with(logger);

    healthcheck_route
}

/// GET /
pub fn homepage_route(
    logger: Log<impl Fn(Info<'_>) + Copy + Send>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let homepage_route = warp::path::end()
        .map(|| {
            warp::http::Response::builder()
                .header("content-type", "text/html")
                .body(format!(
                    "<html>
                <h1>GraphQL Api</h1>
                <div>visit <a href=\"/graphiql\">/graphiql</a></div>
            </html>"
                ))
        })
        .with(logger);

    homepage_route
}
