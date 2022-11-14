use crate::gql::schema::{Context as ResourcesContext, PrivateSchema, PublicSchema};
use juniper::http::GraphQLRequest;
use std::sync::Arc;
use tokio::time::Instant;
use uuid::Uuid;
use warp::Rejection;

pub async fn graphql_public(
    schema: Arc<PublicSchema>,
    ctx: Arc<ResourcesContext>,
    req: GraphQLRequest,
) -> Result<impl warp::Reply, Rejection> {
    let request_uuid = Uuid::new_v4();
    let start = Instant::now();
    let res = req.execute(&schema, &ctx).await;
    log::info!(
        "\nUUID: {:?}\ntime: {:?} milliseconds\noperation: {:?}",
        request_uuid.to_string(),
        start.elapsed().as_millis(),
        req.operation_name().clone().unwrap_or_default()
    );
    let json = warp::reply::json(&res);
    Ok(json)
}

pub async fn graphql_private(
    schema: Arc<PrivateSchema>,
    ctx: Arc<ResourcesContext>,
    req: GraphQLRequest,
    user_id: uuid::Uuid, // authenticated user id calling the gql point
) -> Result<impl warp::Reply, Rejection> {
    {
        let mut lock = ctx.user_id.lock().await;
        *lock = Some(user_id);
        drop(lock);
    }
    let request_uuid = Uuid::new_v4();
    let start = Instant::now();
    let res = req.execute(&schema, &ctx).await;
    log::info!(
        "\nUUID: {:?}\nUserID: {:?}\ntime: {:?} milliseconds\noperation: {:?}",
        request_uuid.to_string(),
        user_id,
        start.elapsed().as_millis(),
        req.operation_name().clone().unwrap_or_default()
    );
    let json = warp::reply::json(&res);
    Ok(json)
}
