use super::{
    error::GqlError,
    models::{Event, NewEvent, UpdateEvent},
};
use crate::{
    auth::Role,
    db::{
        models::{AssetFile, DbEvent, DbTicket},
        sql::{
            db_delete_event_by_id, db_delete_ticket_by_id, db_get_event_by_id,
            db_get_event_by_name, db_get_event_by_slug, db_get_ticket_by_id, db_get_ticket_by_slug,
            db_get_tickets_by_event_id, db_get_user_by_id, db_insert_event, db_insert_ticket,
            db_update_event, db_update_ticket, insert_asset_file,
        },
    },
    gql::{
        error::ValidationError,
        models::{
            EventStatus, NewMintNftsRequest, NewMintNftsResponse, NewTicket, Ticket, UpdateTicket,
        },
        schema::Context as ResourcesContext,
        validations::{
            check_new_ticket_payload, update_event_mutation_payload, update_ticket_mutation_payload,
        },
    },
    grpc::near_api::MintNftsResponse,
};
use slugify::slugify;
use uuid::Uuid;

#[derive(Copy, Clone, Default)]
pub struct PublicMutationRoot;

#[juniper::graphql_object(Context = ResourcesContext)]
impl PublicMutationRoot {
    async fn api_version() -> juniper::FieldResult<&'static str> {
        Ok("v1.0".into())
    }
}
#[derive(Copy, Clone, Default)]
pub struct PrivateMutationRoot;

#[juniper::graphql_object(Context = ResourcesContext)]
impl PrivateMutationRoot {
    async fn api_version() -> juniper::FieldResult<&'static str> {
        Ok("v1.0".into())
    }

    // -------------------------- NFTS ------------------- //

    // seller mint nft tickets
    async fn mint_nfts(
        request: NewMintNftsRequest,
        ctx: &ResourcesContext,
    ) -> Result<NewMintNftsResponse, GqlError> {
        // get the requesting user_id
        let user_id = {
            let lock = ctx.user_id.lock().await;
            let user_id = *lock;
            drop(lock);
            user_id
        }
        .expect("Should have a uuid due to authenticated private gql route");

        // find user in the db
        let db_user = db_get_user_by_id(&ctx.db_client, &user_id)
            .await
            .map_err(|_| {
                GqlError::Validation(ValidationError::new(
                    "user_id",
                    "User not found in the database",
                ))
            })?;

        // check user is a seller
        if !db_user.user_type.eq(&Role::Seller) {
            return Err(GqlError::Validation(ValidationError::new(
                "user_role",
                "User role is not seller. Minting is only allowed for sellers",
            )));
        }

        // get the ticket from db
        let ticket_id = Uuid::parse_str(&request.ticket_id).map_err(|_| GqlError::ParseUUID)?;
        let db_ticket = db_get_ticket_by_id(&ctx.db_client, &ticket_id)
            .await
            .map_err(|_| {
                GqlError::Validation(ValidationError::new(
                    "ticket_id",
                    "Ticket with submitted id does not exist",
                ))
            })?;

        // get the event id for that ticket
        let mut db_event = db_get_event_by_id(&ctx.db_client, &db_ticket.event_id)
            .await
            .map_err(|_| {
                GqlError::Validation(ValidationError::new(
                    "event_id",
                    "Event with submitted id does not exist",
                ))
            })?;

        // make sure the event is in a DRAFT or MINTING states only
        let allowed_states = vec![EventStatus::Draft, EventStatus::Minting];
        if !allowed_states.contains(&db_event.event_status) {
            return Err(GqlError::Validation(ValidationError::new(
                "event_status",
                "Minting could only be applied to events with status DRAFT or MINTING",
            )));
        }

        // check the user is also the event creator
        if !db_user.id.eq(&db_event.created_by_user) {
            return Err(GqlError::Validation(ValidationError::new(
                "event_creator",
                "Event creator and calling user are not the same",
            )));
        }

        // mint the tickets TODO: error handling
        let price = db_ticket
            .price
            .map(|price| price.parse::<f64>())
            .transpose()
            .expect("Price should be parsable!")
            .expect("Price should not be empty!");

        let extra = serde_json::json!({
            "price": price,
        })
        .to_string();

        // media photo and hash
        let media = db_event
            .cover_photo_url
            .clone()
            .expect("Media should not be empty!"); //FIXME: this should be the image from the FE

        let media_hash = sha256::digest(&media);

        let mint_nfts_response = {
            let mut lock = ctx.grpc_near_client.lock().await;
            let mint_nfts_response: MintNftsResponse = lock
                .mint_nfts(
                    db_user.wallet_id,
                    db_ticket.ticket_name,
                    db_ticket.ticket_slug,
                    db_ticket.description.unwrap_or_default(),
                    media,
                    media_hash,
                    db_ticket
                        .quantity_available
                        .expect("Quantity available should not be 0!"),
                    extra,
                    "0".to_string(),
                )
                .await
                .map_err(GqlError::Grpc)?;
            drop(lock);
            mint_nfts_response
        };

        // change the status of the event from DRAFT to MINTING
        if db_event.event_status.eq(&EventStatus::Draft) {
            db_event.event_status = EventStatus::Minting;
            // update the db with the event data
            let _updated_db_event = db_update_event(&ctx.db_client, &db_event)
                .await
                .map_err(GqlError::Database)?;
        }

        // return the tx hash
        Ok(NewMintNftsResponse {
            tx_hash: mint_nfts_response.tx_hash,
        })
    }

    // -------------------------- EVENTS ------------------- //
    async fn register_event(
        new_event: NewEvent,
        ctx: &ResourcesContext,
    ) -> Result<Event, GqlError> {
        // get the requesting user_id
        let user_id = {
            let lock = ctx.user_id.lock().await;
            let user_id = *lock;
            drop(lock);
            user_id
        }
        .expect("Should have a uuid due to authenticated private gql route");

        // check caller is a seller ?
        let db_user = db_get_user_by_id(&ctx.db_client, &user_id)
            .await
            .map_err(|_| {
                GqlError::Validation(ValidationError::new(
                    "user_id",
                    "User not found in the database",
                ))
            })?;

        if !db_user.user_type.eq(&Role::Seller) {
            return Err(GqlError::Validation(ValidationError::new(
                "user type",
                "Calling user is not a seller",
            )));
        }

        // check for unique event slug
        let slug = slugify!(&new_event.event_name, separator = "-");
        if let Ok(_event) = db_get_event_by_slug(&ctx.db_client, &slug).await {
            return Err(GqlError::Validation(ValidationError::new(
                "event_slug",
                "Event with the same slug already exists",
            )));
        }
        // check for unique event name
        if let Ok(_event) = db_get_event_by_name(&ctx.db_client, &new_event.event_name).await {
            return Err(GqlError::Validation(ValidationError::new(
                "event_name",
                "Event with the same name already exists",
            )));
        }

        // save the event into the db (automatically set created date and status to DRAFT)
        let db_event = DbEvent::new(&new_event.event_name, user_id);
        db_insert_event(&ctx.db_client, &db_event)
            .await
            .map_err(GqlError::Database)?;

        Ok(Event::new(db_event, vec![]))
    }

    async fn update_event(
        update_event: UpdateEvent,
        ctx: &ResourcesContext,
    ) -> Result<Event, GqlError> {
        // get the requesting user_id
        let user_id = {
            let lock = ctx.user_id.lock().await;
            let user_id = *lock;
            drop(lock);
            user_id
        }
        .expect("Should have a uuid due to authenticated private gql route");

        // find user in the db
        let db_user = db_get_user_by_id(&ctx.db_client, &user_id)
            .await
            .map_err(|_| {
                GqlError::Validation(ValidationError::new(
                    "user_id",
                    "User not found in the database",
                ))
            })?;

        // get the event id that we want to modify
        let event_id = Uuid::parse_str(&update_event.id).map_err(|_| GqlError::ParseUUID)?;

        // search for event by id
        let mut db_event = db_get_event_by_id(&ctx.db_client, &event_id)
            .await
            .map_err(|_| {
                GqlError::Validation(ValidationError::new(
                    "event_id",
                    "Event with submitted id does not exist",
                ))
            })?;

        // make sure the event is in a DRAFT state only
        if !db_event.event_status.eq(&EventStatus::Draft) {
            return Err(GqlError::Validation(ValidationError::new(
                "event_status",
                "Only event with status DRAFT could be edited",
            )));
        }

        // check caller is event creator ?
        if !db_user.id.eq(&db_event.created_by_user) {
            return Err(GqlError::Validation(ValidationError::new(
                "event_creator",
                "Event creator and calling user are not the same",
            )));
        }

        let cover_photo_base64 = update_event.cover_photo_base64.clone();
        let thumbnail_base64 = update_event.thumbnail_base64.clone();

        // validate and update the event mutation
        let db_event = update_event_mutation_payload(update_event, &mut db_event)?;

        // if uploaded images, send to aws s3
        // TODO: send to worker to do the async sending
        // TODO: do proper error handling
        if let Some(cover_photo) = cover_photo_base64 {
            let path = ctx
                .aws_s3_client
                .upload(None, cover_photo.into_bytes())
                .await
                .expect("failed to upload file to s3");
            db_event.cover_photo_url = Some(ctx.aws_context.get_asset_url(path.clone()));

            // persist the asset in the db and attach it to the event
            let asset_file = AssetFile::new(
                ctx.aws_context.bucket.clone(),
                path,
                None,
                db_event.id.clone(),
            );
            insert_asset_file(&ctx.db_client, &asset_file)
                .await
                .map_err(GqlError::Database)?;
        }

        if let Some(thumbnail) = thumbnail_base64 {
            let path = ctx
                .aws_s3_client
                .upload(None, thumbnail.into_bytes())
                .await
                .expect("failed to upload file to s3");
            db_event.thumbnail_url = Some(ctx.aws_context.get_asset_url(path.clone()));

            // persist the asset in the db and attach it to the event
            let asset_file = AssetFile::new(
                ctx.aws_context.bucket.clone(),
                path,
                None,
                db_event.id.clone(),
            );
            insert_asset_file(&ctx.db_client, &asset_file)
                .await
                .map_err(GqlError::Database)?;
        }

        // update the db with the event data
        let updated_db_event = db_update_event(&ctx.db_client, &db_event)
            .await
            .map_err(GqlError::Database)?;

        // get the related event tickets
        let tickets = db_get_tickets_by_event_id(&ctx.db_client, &Some(updated_db_event.id))
            .await
            .map_err(GqlError::Database)?;

        Ok(Event::new(updated_db_event, tickets))
    }

    async fn delete_event(ctx: &ResourcesContext, id: String) -> Result<bool, GqlError> {
        // get the requesting user_id
        let user_id = {
            let lock = ctx.user_id.lock().await;
            let user_id = *lock;
            drop(lock);
            user_id
        }
        .expect("Should have a uuid due to authenticated private gql route");

        // find user in the db
        let db_user = db_get_user_by_id(&ctx.db_client, &user_id)
            .await
            .map_err(|_| {
                GqlError::Validation(ValidationError::new(
                    "user_id",
                    "User not found in the database",
                ))
            })?;

        // get the event id that we want to delete
        let event_id = Uuid::parse_str(&id).map_err(|_| GqlError::ParseUUID)?;

        let db_event = db_get_event_by_id(&ctx.db_client, &event_id)
            .await
            .map_err(|_| {
                GqlError::Validation(ValidationError::new(
                    "event_id",
                    "Event with submitted id does not exist",
                ))
            })?;

        // make sure the event is in a DRAFT state only. Deleting Minting and Final states not allowed!
        if !db_event.event_status.eq(&EventStatus::Draft) {
            return Err(GqlError::Validation(ValidationError::new(
                "event_status",
                "Only event with status DRAFT could be edited",
            )));
        }

        // check caller is event creator ?
        if !db_user.id.eq(&db_event.created_by_user) {
            return Err(GqlError::Validation(ValidationError::new(
                "event_creator",
                "Event creator and calling user are not the same",
            )));
        }

        // delete event by id
        db_delete_event_by_id(&ctx.db_client, &event_id)
            .await
            .map_err(GqlError::Database)?;
        Ok(true)
    }

    // -------------------------- TICKETS ------------------- //

    async fn add_event_tickets(
        new_tickets: Vec<NewTicket>,
        ctx: &ResourcesContext,
    ) -> Result<Vec<Ticket>, GqlError> {
        // get the requesting user_id
        let user_id = {
            let lock = ctx.user_id.lock().await;
            let user_id = *lock;
            drop(lock);
            user_id
        }
        .expect("Should have a uuid due to authenticated private gql route");

        // find user in the db
        let db_user = db_get_user_by_id(&ctx.db_client, &user_id)
            .await
            .map_err(|_| {
                GqlError::Validation(ValidationError::new(
                    "user_id",
                    "User not found in the database",
                ))
            })?;

        let mut tickets: Vec<Ticket> = vec![];

        for new_ticket in new_tickets.into_iter() {
            // check new ticket data
            check_new_ticket_payload(&new_ticket)?;

            // get ticket event uuid
            let event_id =
                Uuid::parse_str(&new_ticket.event_id).map_err(|_| GqlError::ParseUUID)?;

            // check for event id that the ticket will be attached to
            let db_event = db_get_event_by_id(&ctx.db_client, &event_id)
                .await
                .map_err(|_| {
                    GqlError::Validation(ValidationError::new(
                        "event_id",
                        "Event with submitted id does not exist",
                    ))
                })?;

            // check the user is also the event creator
            if !db_user.id.eq(&db_event.created_by_user) {
                return Err(GqlError::Validation(ValidationError::new(
                    "event_creator",
                    "Event creator and calling user are not the same",
                )));
            }

            // make sure the event is in a DRAFT state only when adding new tickets
            if !db_event.event_status.eq(&EventStatus::Draft) {
                return Err(GqlError::Validation(ValidationError::new(
                    "event_status",
                    "Tickets could only be added to an event with status DRAFT",
                )));
            }

            // check we don't have a ticket with a similar slug and name
            let db_ticket = DbTicket::new(new_ticket, &db_event);
            if let Ok(_ticket) = db_get_ticket_by_slug(&ctx.db_client, &db_ticket.ticket_slug).await
            {
                return Err(GqlError::Validation(ValidationError::new(
                    "ticket_slug",
                    "Ticket with the same slug already exists",
                )));
            }

            // save the ticket into the db
            db_insert_ticket(&ctx.db_client, &db_ticket)
                .await
                .map_err(GqlError::Database)?;

            tickets.push(Ticket::from(db_ticket));
        }

        Ok(tickets)
    }

    async fn delete_event_tickets(
        ctx: &ResourcesContext,
        ids: Vec<String>,
    ) -> Result<bool, GqlError> {
        /*
        // TODO: optimize the deletion in 1 sql statement using IN or using a commit tx ?
        let tickets_to_delete = ids
            .iter()
            .map(|id| Uuid::parse_str(&id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| GqlError::ParseUUID)?;
        */

        // get the requesting user_id
        let user_id = {
            let lock = ctx.user_id.lock().await;
            let user_id = *lock;
            drop(lock);
            user_id
        }
        .expect("Should have a uuid due to authenticated private gql route");

        // find user in the db
        let db_user = db_get_user_by_id(&ctx.db_client, &user_id)
            .await
            .map_err(|_| {
                GqlError::Validation(ValidationError::new(
                    "user_id",
                    "User not found in the database",
                ))
            })?;

        // loop over ticket ids and delete them one by one
        for id in ids.into_iter() {
            // get the ticket id that we want to delete
            let ticket_id = Uuid::parse_str(&id).map_err(|_| GqlError::ParseUUID)?;

            // get ticket data
            let db_ticket = db_get_ticket_by_id(&ctx.db_client, &ticket_id)
                .await
                .map_err(|_| {
                    GqlError::Validation(ValidationError::new(
                        "ticket_id",
                        "Ticket with submitted id does not exist",
                    ))
                })?;

            // get the associated db event
            let db_event = db_get_event_by_id(&ctx.db_client, &db_ticket.event_id)
                .await
                .map_err(|_| {
                    GqlError::Validation(ValidationError::new(
                        "event_ticket_id",
                        "Ticket with event id does not exist",
                    ))
                })?;

            // make sure the event is in a DRAFT state only when deleting tickets
            if !db_event.event_status.eq(&EventStatus::Draft) {
                return Err(GqlError::Validation(ValidationError::new(
                    "event_status",
                    "Tickets could only be deleted for an event with status DRAFT",
                )));
            }

            // check the user is also the event creator
            if !db_user.id.eq(&db_event.created_by_user) {
                return Err(GqlError::Validation(ValidationError::new(
                    "event_creator",
                    "Event creator and calling user are not the same",
                )));
            }

            // delete ticket by id
            db_delete_ticket_by_id(&ctx.db_client, &ticket_id)
                .await
                .map_err(GqlError::Database)?;
        }

        Ok(true)
    }

    async fn update_event_tickets(
        update_tickets: Vec<UpdateTicket>,
        ctx: &ResourcesContext,
    ) -> Result<Vec<Ticket>, GqlError> {
        // get the requesting user_id
        let user_id = {
            let lock = ctx.user_id.lock().await;
            let user_id = *lock;
            drop(lock);
            user_id
        }
        .expect("Should have a uuid due to authenticated private gql route");

        // find user in the db
        let db_user = db_get_user_by_id(&ctx.db_client, &user_id)
            .await
            .map_err(|_| {
                GqlError::Validation(ValidationError::new(
                    "user_id",
                    "User not found in the database",
                ))
            })?;

        let mut tickets: Vec<Ticket> = vec![];

        for update_ticket in update_tickets.into_iter() {
            // get ticket uuid
            let ticket_id = Uuid::parse_str(&update_ticket.id).map_err(|_| GqlError::ParseUUID)?;

            // check we have a db ticket with such uuid
            let mut db_ticket = db_get_ticket_by_id(&ctx.db_client, &ticket_id)
                .await
                .map_err(|_| {
                    GqlError::Validation(ValidationError::new(
                        "ticket_id",
                        "Ticket with submitted id does not exist",
                    ))
                })?;

            // get the associated db event
            let db_event = db_get_event_by_id(&ctx.db_client, &db_ticket.event_id)
                .await
                .map_err(|_| {
                    GqlError::Validation(ValidationError::new(
                        "event_ticket_id",
                        "Ticket with event id does not exist",
                    ))
                })?;

            // make sure the event is in a DRAFT state only when editing tickets
            if !db_event.event_status.eq(&EventStatus::Draft) {
                return Err(GqlError::Validation(ValidationError::new(
                    "event_status",
                    "Tickets could only be edited for an event with status DRAFT",
                )));
            }

            // check the user is also the event creator
            if !db_user.id.eq(&db_event.created_by_user) {
                return Err(GqlError::Validation(ValidationError::new(
                    "event_creator",
                    "Event creator and calling user are not the same",
                )));
            }

            // validate and update the ticket mutation payload
            let db_ticket =
                update_ticket_mutation_payload(update_ticket, &db_event, &mut db_ticket)?;

            // update the db with the ticket data
            let updated_db_ticket = db_update_ticket(&ctx.db_client, &db_ticket)
                .await
                .map_err(GqlError::Database)?;

            tickets.push(Ticket::from(updated_db_ticket));
        }

        Ok(tickets)
    }
}
