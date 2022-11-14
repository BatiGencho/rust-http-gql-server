use super::models::{Event, EventFilter, User};
use crate::{
    db::sql::{db_get_events, db_get_tickets_by_event_id, db_get_user_by_id, db_get_users},
    gql::{error::GqlError, schema::Context as ResourcesContext},
};
use uuid::Uuid;

#[derive(Copy, Clone, Default)]
pub struct PublicQueryRoot;

#[juniper::graphql_object(Context = ResourcesContext)]
impl PublicQueryRoot {
    async fn api_version() -> juniper::FieldResult<&'static str> {
        Ok("v1.0".into())
    }

    async fn events(
        ctx: &ResourcesContext,
        id: Option<String>,
        event_slug: Option<String>,
        filter: Option<EventFilter>,
    ) -> Result<Vec<Event>, GqlError> {
        let event_id = id
            .map(|s| Uuid::parse_str(&s))
            .transpose()
            .map_err(|_| GqlError::ParseUUID)?;

        let tickets = db_get_tickets_by_event_id(&ctx.db_client, &event_id)
            .await
            .map_err(GqlError::Database)?;

        let events: Vec<Event> = db_get_events(&ctx.db_client, event_id, event_slug, filter)
            .await
            .map_err(GqlError::Database)?
            .into_iter()
            .map(|event| {
                let tickets = tickets
                    .iter()
                    .cloned()
                    .filter(|ticket| ticket.event_id.eq(&event.id))
                    .collect::<Vec<_>>();
                Event::new(event, tickets)
            })
            .collect();

        Ok(events)
    }
}

#[derive(Copy, Clone, Default)]
pub struct PrivateQueryRoot;

#[juniper::graphql_object(Context = ResourcesContext)]
impl PrivateQueryRoot {
    async fn api_version() -> juniper::FieldResult<&'static str> {
        Ok("v1.0".into())
    }

    async fn me(ctx: &ResourcesContext) -> Result<User, GqlError> {
        let user_id = {
            let guard = ctx.user_id.lock().await;
            let user_id = guard.ok_or(GqlError::UnexpectedInternal)?;
            drop(guard);
            user_id
        };

        let user = db_get_user_by_id(&ctx.db_client, &user_id)
            .await
            .map_err(GqlError::Database)?;
        Ok(User::from(user))
    }

    async fn users(ctx: &ResourcesContext, id: Option<String>) -> Result<Vec<User>, GqlError> {
        let id = id
            .map(|s| Uuid::parse_str(&s))
            .transpose()
            .map_err(|_| GqlError::ParseUUID)?;

        let users = db_get_users(&ctx.db_client, &id)
            .await
            .map_err(GqlError::Database)?
            .into_iter()
            .map(User::from)
            .collect();
        Ok(users)
    }
}
