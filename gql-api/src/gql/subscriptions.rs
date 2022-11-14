use super::models::{Event, EventFilter};
use crate::{db::sql::db_get_events, gql::schema::Context as ResourcesContext};
use std::pin::Pin;
use uuid::Uuid;

type EventStream = Pin<Box<dyn futures::Stream<Item = Vec<Event>> + Send>>;

#[derive(Copy, Clone, Default)]
pub struct PublicSubscriptionRoot;

#[juniper::graphql_subscription(Context = ResourcesContext)]
impl PublicSubscriptionRoot {
    async fn event_sub(ctx: &ResourcesContext, id: Option<String>) -> EventStream {
        let id = id
            .map(|s| Uuid::parse_str(&s))
            .transpose()
            .expect("Bad uuid");

        let events = db_get_events(&ctx.db_client, id, None, Some(EventFilter::All))
            .await
            .unwrap()
            .into_iter()
            .map(|event| Event::new(event, vec![]))
            .collect();
        Box::pin(futures::stream::once(futures::future::ready(events)))
    }
}

#[derive(Copy, Clone, Default)]
pub struct PrivateSubscriptionRoot;

#[juniper::graphql_subscription(Context = ResourcesContext)]
impl PrivateSubscriptionRoot {
    async fn event_sub(ctx: &ResourcesContext, id: Option<String>) -> EventStream {
        let id = id
            .map(|s| Uuid::parse_str(&s))
            .transpose()
            .expect("Bad uuid");

        let events = db_get_events(&ctx.db_client, id, None, Some(EventFilter::All))
            .await
            .unwrap()
            .into_iter()
            .map(|event| Event::new(event, vec![]))
            .collect();
        Box::pin(futures::stream::once(futures::future::ready(events)))
    }
}
