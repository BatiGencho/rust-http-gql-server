use crate::gql::{
    mutations::{PrivateMutationRoot, PublicMutationRoot},
    quiries::{PrivateQueryRoot, PublicQueryRoot},
    subscriptions::{PrivateSubscriptionRoot, PublicSubscriptionRoot},
};
use juniper::RootNode;
use std::{convert::Infallible, sync::Arc};
use warp::Filter;

pub fn with_public_gql_schema(
    gql_schema: Arc<RootNode<'_, PublicQueryRoot, PublicMutationRoot, PublicSubscriptionRoot>>,
) -> impl warp::Filter<
    Extract = (Arc<RootNode<'_, PublicQueryRoot, PublicMutationRoot, PublicSubscriptionRoot>>,),
    Error = Infallible,
> + Clone {
    warp::any().map(move || Arc::clone(&gql_schema))
}

pub fn with_private_gql_schema(
    gql_schema: Arc<RootNode<'_, PrivateQueryRoot, PrivateMutationRoot, PrivateSubscriptionRoot>>,
) -> impl warp::Filter<
    Extract = (Arc<RootNode<'_, PrivateQueryRoot, PrivateMutationRoot, PrivateSubscriptionRoot>>,),
    Error = Infallible,
> + Clone {
    warp::any().map(move || Arc::clone(&gql_schema))
}
