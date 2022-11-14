use std::{net::SocketAddr, sync::Arc};

use super::{
    filters::{with_private_gql_schema, with_public_gql_schema},
    handlers::{
        graphql_private as graphql_private_handler, graphql_public as graphql_public_handler,
    },
    schema::{Context as ResourcesContext, PrivateSchema, PublicSchema},
};
use crate::{
    auth::Role,
    filters::{with_auth, with_resources_context},
};
use juniper::http::graphiql::graphiql_source;
use warp::{
    self,
    log::{Info, Log},
    Filter,
};

/// POST /graphql/public
pub fn graphql_public_route(
    resources_ctx: Arc<ResourcesContext>,
    gql_schema: Arc<PublicSchema>,
    logger: Log<impl Fn(Info<'_>) + Copy + Send + 'static>,
) -> impl Filter<Extract = impl warp::Reply + 'static, Error = warp::Rejection> + Clone + 'static {
    let graphql_route = warp::post()
        .and(warp::path!("api" / "v1" / "graphql" / "public"))
        .and(with_public_gql_schema(gql_schema))
        .and(with_resources_context(resources_ctx))
        .and(warp::body::json())
        .and_then(graphql_public_handler)
        .with(logger);
    graphql_route
}

/// POST /graphql/private
pub fn graphql_private_route(
    resources_ctx: Arc<ResourcesContext>,
    gql_schema: Arc<PrivateSchema>,
    logger: Log<impl Fn(Info<'_>) + Copy + Send + 'static>,
) -> impl Filter<Extract = impl warp::Reply + 'static, Error = warp::Rejection> + Clone + 'static {
    let graphql_route = warp::post()
        .and(warp::path!("api" / "v1" / "graphql" / "private"))
        .and(with_private_gql_schema(gql_schema))
        .and(with_resources_context(resources_ctx))
        .and(warp::body::json())
        .and(with_auth(vec![
            Role::Admin,
            Role::Buyer,
            Role::Seller,
            Role::SuperAdmin,
        ]))
        .and_then(graphql_private_handler)
        .with(logger);
    graphql_route
}

/// POST /graphiql
pub fn public_graphiql_route(
    server_addr: SocketAddr,
    logger: Log<impl Fn(Info<'_>) + Copy + Send>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let gql_endpoint = format!(
        "http://localhost:{}/api/v1/graphql/public",
        server_addr.port()
    );
    let graphiql_route = warp::get()
        .and(warp::path!("api" / "v1" / "graphiql"))
        .map(move || {
            warp::reply::html(graphiql_source(
                &gql_endpoint,
                None, //Some(format!("ws://{}/api/v1/subscriptions", server_addr.to_string()).as_str())
            ))
        })
        .with(logger);

    graphiql_route
}
