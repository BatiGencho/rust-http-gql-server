use super::models::{
    AssetFile, DbBuyerRecoverySession, DbBuyerSignupSession, DbEvent, DbSession, DbTicket,
    DbTicketReservation, DbUser,
};
use crate::gql::models::EventFilter;
use chrono::{Duration, NaiveDateTime, Utc};
use std::borrow::Cow;
use std::convert::TryFrom;
use tokio_postgres::types::ToSql;
use tokio_postgres::Client;

lazy_static::lazy_static! {

    // events table
    pub static ref EVENTS_TABLE: String = "events".to_string();
    pub static ref EVENTS_TABLE_FIELDS: String = "id,
                                                event_name,
                                                event_slug,
                                                start_date,
                                                end_date,
                                                entry_time,
                                                created_at,
                                                description,
                                                is_virtual,
                                                is_featured,
                                                venue_name,
                                                venue_location,
                                                cover_photo_url,
                                                thumbnail_url,
                                                event_status,
                                                created_by_user".to_string();

    // tickets table
    pub static ref TICKETS_TABLE: String = "tickets".to_string();
    pub static ref TICKETS_TABLE_FIELDS: String = "id,
                                                    created_at,
                                                    ticket_name,
                                                    ticket_slug,
                                                    description,
                                                    price,
                                                    max_release_price,
                                                    quantity_available,
                                                    min_purchase_quantity,
                                                    max_purchase_quantity,
                                                    allow_transfers,
                                                    event_id".to_string();

    // users table
    pub static ref USERS_TABLE: String = "users".to_string();
    pub static ref USERS_TABLE_FIELDS: String = "id,
                                                name,
                                                username,
                                                phone_number,
                                                email,
                                                password,
                                                encrypted_secret_key,
                                                created_at,
                                                wallet_id,
                                                wallet_balance,
                                                user_type,
                                                user_status".to_string();

    // buyer login sessions table
    pub static ref SESSIONS_TABLE: String = "sessions".to_string();
    pub static ref SESSIONS_TABLE_FIELDS: String = "id,
                                                    expires_at,
                                                    login_code,
                                                    is_used,
                                                    user_id".to_string();

    // buyer signup sessions table
    pub static ref BUYER_SIGNUP_SESSIONS_TABLE: String = "buyer_signup_sessions".to_string();
    pub static ref BUYER_SIGNUP_SESSIONS_TABLE_FIELDS: String = "id,
                                                                created_at,
                                                                verification_code,
                                                                phone_number,
                                                                is_verified".to_string();

    // buyer recovery sessions table
    pub static ref BUYER_RECOVERY_SESSIONS_TABLE: String = "buyer_recovery_sessions".to_string();
    pub static ref BUYER_RECOVERY_SESSIONS_TABLE_FIELDS: String = "id,
                                                                    created_at,
                                                                    recovery_code,
                                                                    phone_number,
                                                                    is_recovered,
                                                                    created_by_user".to_string();

    // ticket reservations table
    pub static ref TICKET_RESERVATIONS_TABLE: String = "ticket_reservations".to_string();
    pub static ref TICKET_RESERVATIONS_TABLE_FIELDS: String = "id,
                                                                created_at,
                                                                verification_code,
                                                                event_id,
                                                                ticket_id,
                                                                user_id".to_string();

    // s3 files table
    pub static ref ASSET_FILES_TABLE: String = "asset_files".to_string();
    pub static ref ASSET_FILES_SELECT_FIELDS: String = "id,
                                                    s3_bucket,
                                                    s3_absolute_key,
                                                    ipfs_hash,
                                                    event_id
                                                    ".to_string();
}

pub async fn db_insert_event(
    db_client: &Client,
    new_event: &DbEvent,
) -> Result<u64, tokio_postgres::Error> {
    let insert_query = format!(
        "INSERT INTO {} 
                ({})
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)",
        *EVENTS_TABLE, *EVENTS_TABLE_FIELDS
    );
    let create_event_statement = db_client.prepare(&insert_query).await?;

    let res_create_event = db_client
        .execute(
            &create_event_statement,
            &[
                &new_event.id,
                &new_event.event_name,
                &new_event.event_slug,
                &new_event.start_date,
                &new_event.end_date,
                &new_event.entry_time,
                &new_event.created_at,
                &new_event.description,
                &new_event.is_virtual,
                &new_event.is_featured,
                &new_event.venue_name,
                &new_event.venue_location,
                &new_event.cover_photo_url,
                &new_event.thumbnail_url,
                &(new_event.event_status as i16),
                &new_event.created_by_user,
            ],
        )
        .await;
    res_create_event
}

pub async fn db_update_event(
    db_client: &Client,
    new_event: &DbEvent,
) -> Result<DbEvent, tokio_postgres::Error> {
    let update_query = format!(
        "UPDATE {} 
         SET event_name = $1::VARCHAR,
            event_slug = $2::VARCHAR,
            start_date = $3::TIMESTAMP,
            end_date = $4::TIMESTAMP,
            entry_time = $5::TIMESTAMP,
            description = $6::VARCHAR,
            is_virtual = $7::BOOLEAN,
            is_featured = $8::BOOLEAN,
            venue_name = $9::VARCHAR,
            venue_location = $10::VARCHAR,
            cover_photo_url = $11::VARCHAR,
            thumbnail_url = $12::VARCHAR,
            created_by_user = $13::UUID
         WHERE id = $14::UUID
         RETURNING {}",
        *EVENTS_TABLE, *EVENTS_TABLE_FIELDS
    );

    let update_stmt = db_client.prepare(&update_query).await?;

    let x = db_client
        .query_one(
            &update_stmt,
            &[
                &new_event.event_name,
                &new_event.event_slug,
                &new_event.start_date,
                &new_event.end_date,
                &new_event.entry_time,
                &new_event.description,
                &new_event.is_virtual,
                &new_event.is_featured,
                &new_event.venue_name,
                &new_event.venue_location,
                &new_event.cover_photo_url,
                &new_event.thumbnail_url,
                &new_event.created_by_user,
                &new_event.id,
            ],
        )
        .await?;

    x.try_into()
}

pub async fn db_update_ticket(
    db_client: &Client,
    new_ticket: &DbTicket,
) -> Result<DbTicket, tokio_postgres::Error> {
    let update_query = format!(
        "UPDATE {} 
            SET ticket_name = $1::VARCHAR,
            ticket_slug = $2::VARCHAR,
            description = $3::VARCHAR,
            price = $4::VARCHAR,
            max_release_price = $5::VARCHAR,
            quantity_available = $6::INTEGER,
            min_purchase_quantity = $7::INTEGER,
            max_purchase_quantity = $8::INTEGER,
            allow_transfers = $9::BOOLEAN
         WHERE id = $10::UUID
         RETURNING {}",
        *TICKETS_TABLE, *TICKETS_TABLE_FIELDS
    );

    let update_stmt = db_client.prepare(&update_query).await?;

    let x = db_client
        .query_one(
            &update_stmt,
            &[
                &new_ticket.ticket_name,
                &new_ticket.ticket_slug,
                &new_ticket.description,
                &new_ticket.price,
                &new_ticket.max_release_price,
                &new_ticket.quantity_available,
                &new_ticket.min_purchase_quantity,
                &new_ticket.max_purchase_quantity,
                &new_ticket.allow_transfers,
                &new_ticket.id,
            ],
        )
        .await?;

    x.try_into()
}

pub async fn db_insert_ticket(
    db_client: &Client,
    db_ticket: &DbTicket,
) -> Result<u64, tokio_postgres::Error> {
    let insert_query = format!(
        "INSERT INTO {} 
                ({})
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        *TICKETS_TABLE, *TICKETS_TABLE_FIELDS
    );
    let insert_stmt = db_client.prepare(&insert_query).await?;
    let res_ticket = db_client
        .execute(
            &insert_stmt,
            &[
                &db_ticket.id,
                &db_ticket.created_at,
                &db_ticket.ticket_name,
                &db_ticket.ticket_slug,
                &db_ticket.description,
                &db_ticket.price,
                &db_ticket.max_release_price,
                &db_ticket.quantity_available,
                &db_ticket.min_purchase_quantity,
                &db_ticket.max_purchase_quantity,
                &db_ticket.allow_transfers,
                &db_ticket.event_id,
            ],
        )
        .await;
    res_ticket
}

pub async fn db_insert_user(
    db_client: &Client,
    new_user: &DbUser,
) -> Result<u64, tokio_postgres::Error> {
    let insert_query = format!(
        "INSERT INTO {} 
                ({})
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        *USERS_TABLE, *USERS_TABLE_FIELDS
    );
    let create_user_statement = db_client.prepare(&insert_query).await?;

    let res_user_event = db_client
        .execute(
            &create_user_statement,
            &[
                &new_user.id,
                &new_user.name,
                &new_user.username,
                &new_user.phone_number,
                &new_user.email,
                &new_user.password,
                &new_user.encrypted_secret_key,
                &new_user.created_at,
                &new_user.wallet_id,
                &new_user.wallet_balance,
                &(new_user.user_type as i16),
                &(new_user.user_status as i16),
            ],
        )
        .await;
    res_user_event
}

pub async fn db_insert_session(
    db_client: &Client,
    new_session: &DbSession,
) -> Result<u64, tokio_postgres::Error> {
    let insert_query = format!(
        "INSERT INTO {} 
                ({})
            VALUES ($1, $2, $3, $4, $5)",
        *SESSIONS_TABLE, *SESSIONS_TABLE_FIELDS
    );
    let create_session_statement = db_client.prepare(&insert_query).await?;

    let res_session_event = db_client
        .execute(
            &create_session_statement,
            &[
                &new_session.id,
                &new_session.expires_at,
                &new_session.login_code,
                &new_session.is_used,
                &new_session.user_id,
            ],
        )
        .await;
    res_session_event
}

pub async fn db_insert_buyer_recovery_session(
    db_client: &Client,
    db_buyer_recovery_session: &DbBuyerRecoverySession,
) -> Result<u64, tokio_postgres::Error> {
    let insert_query = format!(
        "INSERT INTO {} 
                ({})
            VALUES ($1, $2, $3, $4, $5, $6)",
        *BUYER_RECOVERY_SESSIONS_TABLE, *BUYER_RECOVERY_SESSIONS_TABLE_FIELDS
    );
    let create_buyer_recovery_session_statement = db_client.prepare(&insert_query).await?;

    let res = db_client
        .execute(
            &create_buyer_recovery_session_statement,
            &[
                &db_buyer_recovery_session.id,
                &db_buyer_recovery_session.created_at,
                &db_buyer_recovery_session.recovery_code,
                &db_buyer_recovery_session.phone_number,
                &db_buyer_recovery_session.is_recovered,
                &db_buyer_recovery_session.created_by_user,
            ],
        )
        .await;
    res
}

pub async fn db_insert_buyer_signup_session(
    db_client: &Client,
    db_buyer_signup_session: &DbBuyerSignupSession,
) -> Result<u64, tokio_postgres::Error> {
    let insert_query = format!(
        "INSERT INTO {} 
                ({})
            VALUES ($1, $2, $3, $4, $5)",
        *BUYER_SIGNUP_SESSIONS_TABLE, *BUYER_SIGNUP_SESSIONS_TABLE_FIELDS
    );
    let create_buyer_signup_session_statement = db_client.prepare(&insert_query).await?;

    let res_session_event = db_client
        .execute(
            &create_buyer_signup_session_statement,
            &[
                &db_buyer_signup_session.id,
                &db_buyer_signup_session.created_at,
                &db_buyer_signup_session.verification_code,
                &db_buyer_signup_session.phone_number,
                &db_buyer_signup_session.is_verified,
            ],
        )
        .await;
    res_session_event
}

pub async fn db_update_buyer_signup_session(
    db_client: &Client,
    buyer_signup_session: &DbBuyerSignupSession,
) -> Result<DbBuyerSignupSession, tokio_postgres::Error> {
    let update_query = format!(
        "UPDATE {} 
            SET verification_code = $1::VARCHAR,
            phone_number = $2::VARCHAR,
            is_verified = $3::BOOLEAN
         WHERE id = $4::UUID
         RETURNING {}",
        *BUYER_SIGNUP_SESSIONS_TABLE, *BUYER_SIGNUP_SESSIONS_TABLE_FIELDS
    );

    let update_stmt = db_client.prepare(&update_query).await?;

    let x = db_client
        .query_one(
            &update_stmt,
            &[
                &buyer_signup_session.verification_code,
                &buyer_signup_session.phone_number,
                &buyer_signup_session.is_verified,
                &buyer_signup_session.id,
            ],
        )
        .await?;

    x.try_into()
}

pub async fn db_update_buyer_recovery_session(
    db_client: &Client,
    buyer_recovery_session: &DbBuyerRecoverySession,
) -> Result<DbBuyerRecoverySession, tokio_postgres::Error> {
    let update_query = format!(
        "UPDATE {} 
            SET recovery_code = $1::VARCHAR,
            phone_number = $2::VARCHAR,
            is_recovered = $3::BOOLEAN
         WHERE id = $4::UUID
         RETURNING {}",
        *BUYER_RECOVERY_SESSIONS_TABLE, *BUYER_RECOVERY_SESSIONS_TABLE_FIELDS
    );

    let update_stmt = db_client.prepare(&update_query).await?;

    let x = db_client
        .query_one(
            &update_stmt,
            &[
                &buyer_recovery_session.recovery_code,
                &buyer_recovery_session.phone_number,
                &buyer_recovery_session.is_recovered,
                &buyer_recovery_session.id,
            ],
        )
        .await?;

    x.try_into()
}

pub async fn db_get_buyer_signup_session_by_id(
    db_client: &Client,
    session_id: &uuid::Uuid,
) -> Result<DbBuyerSignupSession, tokio_postgres::Error> {
    let query = format!(
        "SELECT {} FROM {} WHERE id = $1::UUID",
        *BUYER_SIGNUP_SESSIONS_TABLE_FIELDS, *BUYER_SIGNUP_SESSIONS_TABLE
    );
    let query_values: Vec<&(dyn ToSql + Sync)> = vec![&session_id];
    let row: tokio_postgres::Row = db_client.query_one(&query, query_values.as_slice()).await?;
    DbBuyerSignupSession::try_from(row)
}

pub async fn db_get_buyer_recovery_session_by_id(
    db_client: &Client,
    session_id: &uuid::Uuid,
) -> Result<DbBuyerRecoverySession, tokio_postgres::Error> {
    let query = format!(
        "SELECT {} FROM {} WHERE id = $1::UUID",
        *BUYER_RECOVERY_SESSIONS_TABLE_FIELDS, *BUYER_RECOVERY_SESSIONS_TABLE
    );
    let query_values: Vec<&(dyn ToSql + Sync)> = vec![&session_id];
    let row: tokio_postgres::Row = db_client.query_one(&query, query_values.as_slice()).await?;
    DbBuyerRecoverySession::try_from(row)
}

pub async fn db_get_session_by_login_code(
    db_client: &Client,
    login_code: &str,
) -> Result<DbSession, tokio_postgres::Error> {
    let query = format!(
        "SELECT {} FROM {} WHERE login_code = $1::VARCHAR",
        *SESSIONS_TABLE_FIELDS, *SESSIONS_TABLE
    );
    let query_values: Vec<&(dyn ToSql + Sync)> = vec![&login_code];
    let row: tokio_postgres::Row = db_client.query_one(&query, query_values.as_slice()).await?;
    DbSession::try_from(row)
}

pub async fn db_update_session_info(
    db_client: &Client,
    session_id: &uuid::Uuid,
    user_id: &uuid::Uuid,
    is_used: bool,
) -> Result<u64, tokio_postgres::Error> {
    let res = db_client
        .execute(
            &format!(
                "UPDATE {} SET is_used = $1::BOOLEAN, user_id = $2::UUID WHERE id = $3::UUID",
                *SESSIONS_TABLE
            ),
            &[&is_used, &user_id, &session_id],
        )
        .await;
    res
}

pub async fn db_get_events(
    db_client: &Client,
    event_id: Option<uuid::Uuid>,
    event_slug: Option<String>,
    event_filter: Option<EventFilter>,
) -> Result<Vec<DbEvent>, tokio_postgres::Error> {
    let mut query = format!(
        "SELECT {} FROM {} WHERE ($1::UUID is NULL OR id = $1::UUID) AND ($2::VARCHAR is NULL OR event_slug = $2::VARCHAR)",
        *EVENTS_TABLE_FIELDS, *EVENTS_TABLE
    );
    let mut query_values: Vec<&(dyn ToSql + Sync)> = vec![&event_id, &event_slug];
    if let Some(event_filter) = event_filter {
        match event_filter {
            EventFilter::Featured => {
                query = format!("{} AND (is_featured = $2::BOOLEAN)", query);
                query_values.extend_from_slice(&[&true]);
            }
            EventFilter::NoneFeatured => {
                query = format!("{} AND (is_featured = $2::BOOLEAN)", query);
                query_values.extend_from_slice(&[&false]);
            }
            EventFilter::All => (),
        }
    }
    let rows: Vec<tokio_postgres::Row> = db_client.query(&query, query_values.as_slice()).await?;
    let events: Result<Vec<_>, _> = rows.into_iter().map(|r| DbEvent::try_from(r)).collect();
    events
}

pub async fn db_get_user_by_id(
    db_client: &Client,
    user_id: &uuid::Uuid,
) -> Result<DbUser, tokio_postgres::Error> {
    let query = format!(
        "SELECT {} FROM {} WHERE ($1::UUID is NULL OR id = $1::UUID)",
        *USERS_TABLE_FIELDS, *USERS_TABLE
    );
    let query_values: Vec<&(dyn ToSql + Sync)> = vec![&user_id];
    let row: tokio_postgres::Row = db_client.query_one(&query, query_values.as_slice()).await?;
    DbUser::try_from(row)
}

pub async fn db_get_users_by_username(
    db_client: &Client,
    username: &str,
) -> Result<Vec<DbUser>, tokio_postgres::Error> {
    let query = format!(
        "SELECT {} FROM {} WHERE username = $1::VARCHAR",
        *USERS_TABLE_FIELDS, *USERS_TABLE
    );
    let query_values: Vec<&(dyn ToSql + Sync)> = vec![&username];
    let rows: Vec<tokio_postgres::Row> = db_client.query(&query, query_values.as_slice()).await?;
    let users: Result<Vec<_>, _> = rows.into_iter().map(|r| DbUser::try_from(r)).collect();
    users
}

pub async fn db_get_user_by_username(
    db_client: &Client,
    username: &str,
) -> Result<DbUser, tokio_postgres::Error> {
    let query = format!(
        "SELECT {} FROM {} WHERE username = $1::VARCHAR",
        *USERS_TABLE_FIELDS, *USERS_TABLE
    );
    let query_values: Vec<&(dyn ToSql + Sync)> = vec![&username];
    let row: tokio_postgres::Row = db_client.query_one(&query, query_values.as_slice()).await?;
    DbUser::try_from(row)
}

pub async fn db_get_user_by_email(
    db_client: &Client,
    email: &str,
) -> Result<DbUser, tokio_postgres::Error> {
    let query = format!(
        "SELECT {} FROM {} WHERE email = $1::VARCHAR",
        *USERS_TABLE_FIELDS, *USERS_TABLE
    );
    let query_values: Vec<&(dyn ToSql + Sync)> = vec![&email];
    let row: tokio_postgres::Row = db_client.query_one(&query, query_values.as_slice()).await?;
    DbUser::try_from(row)
}

pub async fn db_get_user_by_name(
    db_client: &Client,
    name: &str,
) -> Result<DbUser, tokio_postgres::Error> {
    let query = format!(
        "SELECT {} FROM {} WHERE name = $1::VARCHAR",
        *USERS_TABLE_FIELDS, *USERS_TABLE
    );
    let query_values: Vec<&(dyn ToSql + Sync)> = vec![&name];
    let row: tokio_postgres::Row = db_client.query_one(&query, query_values.as_slice()).await?;
    DbUser::try_from(row)
}

pub async fn db_get_user_by_phone_number(
    db_client: &Client,
    phone_number: &str,
) -> Result<DbUser, tokio_postgres::Error> {
    let query = format!(
        "SELECT {} FROM {} WHERE phone_number = $1::VARCHAR",
        *USERS_TABLE_FIELDS, *USERS_TABLE
    );
    let query_values: Vec<&(dyn ToSql + Sync)> = vec![&phone_number];
    let row: tokio_postgres::Row = db_client.query_one(&query, query_values.as_slice()).await?;
    DbUser::try_from(row)
}

pub async fn db_get_user_by_wallet_id(
    db_client: &Client,
    wallet_id: &str,
) -> Result<DbUser, tokio_postgres::Error> {
    let query = format!(
        "SELECT {} FROM {} WHERE wallet_id = $1::VARCHAR",
        *USERS_TABLE_FIELDS, *USERS_TABLE
    );
    let query_values: Vec<&(dyn ToSql + Sync)> = vec![&wallet_id];
    let row: tokio_postgres::Row = db_client.query_one(&query, query_values.as_slice()).await?;
    DbUser::try_from(row)
}

pub async fn db_get_users(
    db_client: &Client,
    user_id: &Option<uuid::Uuid>,
) -> Result<Vec<DbUser>, tokio_postgres::Error> {
    let query = format!(
        "SELECT {} FROM {} WHERE ($1::UUID is NULL OR id = $1::UUID)",
        *USERS_TABLE_FIELDS, *USERS_TABLE
    );
    let query_values: Vec<&(dyn ToSql + Sync)> = vec![&user_id];
    let rows: Vec<tokio_postgres::Row> = db_client.query(&query, query_values.as_slice()).await?;
    let users: Result<Vec<_>, _> = rows.into_iter().map(|r| DbUser::try_from(r)).collect();
    users
}

pub async fn db_get_event_by_name(
    db_client: &Client,
    event_name: &str,
) -> Result<DbEvent, tokio_postgres::Error> {
    let query = format!(
        "SELECT {} FROM {} WHERE event_name = $1::VARCHAR",
        *EVENTS_TABLE_FIELDS, *EVENTS_TABLE
    );
    let query_values: Vec<&(dyn ToSql + Sync)> = vec![&event_name];
    let row: tokio_postgres::Row = db_client.query_one(&query, query_values.as_slice()).await?;
    DbEvent::try_from(row)
}

pub async fn db_get_event_by_slug(
    db_client: &Client,
    event_slug: &str,
) -> Result<DbEvent, tokio_postgres::Error> {
    let query = format!(
        "SELECT {} FROM {} WHERE event_slug = $1::VARCHAR",
        *EVENTS_TABLE_FIELDS, *EVENTS_TABLE
    );
    let query_values: Vec<&(dyn ToSql + Sync)> = vec![&event_slug];
    let row: tokio_postgres::Row = db_client.query_one(&query, query_values.as_slice()).await?;
    DbEvent::try_from(row)
}

pub async fn db_get_event_by_id(
    db_client: &Client,
    id: &uuid::Uuid,
) -> Result<DbEvent, tokio_postgres::Error> {
    let query = format!(
        "SELECT {} FROM {} WHERE id = $1::UUID",
        *EVENTS_TABLE_FIELDS, *EVENTS_TABLE
    );
    let query_values: Vec<&(dyn ToSql + Sync)> = vec![&id];
    let row: tokio_postgres::Row = db_client.query_one(&query, query_values.as_slice()).await?;
    DbEvent::try_from(row)
}

pub async fn db_delete_event_by_id(
    db_client: &Client,
    id: &uuid::Uuid,
) -> Result<u64, tokio_postgres::Error> {
    let res = db_client
        .execute(
            &format!("DELETE FROM {} WHERE id = $1::UUID", *EVENTS_TABLE),
            &[&id],
        )
        .await;
    res
}

pub async fn db_delete_ticket_by_id(
    db_client: &Client,
    id: &uuid::Uuid,
) -> Result<u64, tokio_postgres::Error> {
    let res = db_client
        .execute(
            &format!("DELETE FROM {} WHERE id = $1::UUID", *TICKETS_TABLE),
            &[&id],
        )
        .await;
    res
}

pub async fn db_get_tickets_by_event_id(
    db_client: &Client,
    event_id: &Option<uuid::Uuid>,
) -> Result<Vec<DbTicket>, tokio_postgres::Error> {
    let query = format!(
        "SELECT {} FROM {} WHERE ($1::UUID is NULL OR event_id = $1::UUID)",
        *TICKETS_TABLE_FIELDS, *TICKETS_TABLE
    );
    let query_values: Vec<&(dyn ToSql + Sync)> = vec![&event_id];
    let rows: Vec<tokio_postgres::Row> = db_client.query(&query, query_values.as_slice()).await?;
    let tickets: Result<Vec<_>, _> = rows.into_iter().map(|r| DbTicket::try_from(r)).collect();
    tickets
}

pub async fn db_get_ticket_by_slug(
    db_client: &Client,
    ticket_slug: &str,
) -> Result<DbTicket, tokio_postgres::Error> {
    let query = format!(
        "SELECT {} FROM {} WHERE ticket_slug = $1::VARCHAR",
        *TICKETS_TABLE_FIELDS, *TICKETS_TABLE
    );
    let query_values: Vec<&(dyn ToSql + Sync)> = vec![&ticket_slug];
    let row: tokio_postgres::Row = db_client.query_one(&query, query_values.as_slice()).await?;
    DbTicket::try_from(row)
}

pub async fn db_get_ticket_by_id(
    db_client: &Client,
    ticket_id: &uuid::Uuid,
) -> Result<DbTicket, tokio_postgres::Error> {
    let query = format!(
        "SELECT {} FROM {} WHERE id = $1::UUID",
        *TICKETS_TABLE_FIELDS, *TICKETS_TABLE
    );
    let query_values: Vec<&(dyn ToSql + Sync)> = vec![&ticket_id];
    let row: tokio_postgres::Row = db_client.query_one(&query, query_values.as_slice()).await?;
    DbTicket::try_from(row)
}

pub async fn db_get_asset_file(
    db_client: &Client,
    id: &uuid::Uuid,
) -> Result<AssetFile, tokio_postgres::Error> {
    let query = format!(
        "SELECT {} FROM {} WHERE id = $1::UUID",
        *ASSET_FILES_SELECT_FIELDS, *ASSET_FILES_TABLE
    );
    let query_values: Vec<&(dyn ToSql + Sync)> = vec![&id];
    let row: tokio_postgres::Row = db_client.query_one(&query, query_values.as_slice()).await?;
    AssetFile::try_from(row)
}

pub async fn db_get_files_for_event(
    db_client: &Client,
    event_id: &uuid::Uuid,
) -> Result<Vec<AssetFile>, tokio_postgres::Error> {
    let query = format!(
        "SELECT {} FROM {} WHERE event_id = $1::UUID",
        *ASSET_FILES_SELECT_FIELDS, *ASSET_FILES_TABLE
    );
    let query_values: Vec<&(dyn ToSql + Sync)> = vec![&event_id];
    let rows: Vec<tokio_postgres::Row> = db_client.query(&query, query_values.as_slice()).await?;

    rows.into_iter().map(|r| AssetFile::try_from(r)).collect()
}

pub async fn update_file_ipfs_hash(
    db_client: &Client,
    id: &uuid::Uuid,
    hash: &String,
) -> Result<AssetFile, tokio_postgres::Error> {
    let update_query = format!(
        "UPDATE {} 
         SET ipfs_hash = $1
         WHERE id = $2 AND ipfs_hash is NULL
         RETURNING {}",
        *ASSET_FILES_TABLE, *ASSET_FILES_SELECT_FIELDS
    );

    let update_stmt = db_client.prepare(&update_query).await?;

    let x = db_client.query_one(&update_stmt, &[&hash, &id]).await?;

    x.try_into()
}

pub async fn insert_asset_file(
    db_client: &Client,
    file: &AssetFile,
) -> Result<AssetFile, tokio_postgres::Error> {
    let insert_query = format!(
        "INSERT INTO {} 
                ({})
            VALUES ($1, $2, $3, $4, $5)",
        *ASSET_FILES_TABLE, *ASSET_FILES_SELECT_FIELDS
    );
    let create_s3_file_stmt = db_client.prepare(&insert_query).await?;

    db_client
        .execute(
            &create_s3_file_stmt,
            &[
                &file.id,
                &file.s3_bucket,
                &file.s3_absolute_key,
                &file.ipfs_hash,
                &file.event_id,
            ],
        )
        .await?;

    Ok(file.clone())
}

pub async fn db_insert_ticket_reservation(
    db_client: &Client,
    db_ticket_reservation: &DbTicketReservation,
) -> Result<u64, tokio_postgres::Error> {
    let insert_query = format!(
        "INSERT INTO {}
                ({})
            VALUES ($1, $2, $3, $4, $5, $6)",
        *TICKET_RESERVATIONS_TABLE, *TICKET_RESERVATIONS_TABLE_FIELDS
    );
    let create_statement = db_client.prepare(&insert_query).await?;

    let res_ticket_reservation = db_client
        .execute(
            &create_statement,
            &[
                &db_ticket_reservation.id,
                &db_ticket_reservation.created_at,
                &db_ticket_reservation.verification_code,
                &db_ticket_reservation.event_id,
                &db_ticket_reservation.ticket_id,
                &db_ticket_reservation.user_id,
            ],
        )
        .await;
    res_ticket_reservation
}

/*
pub enum TicketReservationQueryItem {
    VerificationCode(String),
    UserId(i32),
}

pub async fn db_get_ticket_reservation(
    db_client: &Client,
    query_item: TicketReservationQueryItem,
) -> Result<DbTicketReservation, tokio_postgres::Error> {
    let (query, query_values) = match query_item {
        TicketReservationQueryItem::VerificationCode(verification_code) => {
            let query = format!(
                "SELECT {} FROM {} WHERE verification_code = $1::VARCHAR",
                *TICKET_RESERVATIONS_TABLE_FIELDS, *TICKET_RESERVATIONS_TABLE
            );
            let query_values: Vec<Box<(dyn ToSql + Sync)>> = vec![Box::new(verification_code)];
            (query, query_values)
        }
        TicketReservationQueryItem::UserId(user_id) => {
            let query = format!(
                "SELECT {} FROM {} WHERE user_id = $1::INT",
                *TICKET_RESERVATIONS_TABLE_FIELDS, *TICKET_RESERVATIONS_TABLE
            );
            let query_values: Vec<Box<(dyn ToSql + Sync)>> = vec![Box::new(user_id)];
            (query, query_values)
        }
    };
    let vec_of_refs = query_values
        .iter()
        .map(|x| &**x)
        .collect::<Vec<&(dyn ToSql + Sync)>>();

    let row: tokio_postgres::Row = db_client.query_one(&query, vec_of_refs.as_slice()).await?;
    DbTicketReservation::try_from(row)
}
*/

pub async fn db_get_ticket_reservations_by_code(
    db_client: &Client,
    verification_code: &str,
) -> Result<Vec<DbTicketReservation>, tokio_postgres::Error> {
    let query = format!(
        "SELECT {} FROM {} WHERE verification_code = $1::VARCHAR",
        *TICKET_RESERVATIONS_TABLE_FIELDS, *TICKET_RESERVATIONS_TABLE
    );
    let query_values: Vec<&(dyn ToSql + Sync)> = vec![&verification_code];
    let rows: Vec<tokio_postgres::Row> = db_client.query(&query, query_values.as_slice()).await?;
    let reservations: Result<Vec<_>, _> = rows
        .into_iter()
        .map(|r| DbTicketReservation::try_from(r))
        .collect();
    reservations
}

pub async fn db_get_ticket_reservations_by_user_id(
    db_client: &Client,
    user_id: &uuid::Uuid,
) -> Result<Vec<DbTicketReservation>, tokio_postgres::Error> {
    let query = format!(
        "SELECT {} FROM {} WHERE user_id = $1::UUID",
        *TICKET_RESERVATIONS_TABLE_FIELDS, *TICKET_RESERVATIONS_TABLE
    );
    let query_values: Vec<&(dyn ToSql + Sync)> = vec![&user_id];
    let rows: Vec<tokio_postgres::Row> = db_client.query(&query, query_values.as_slice()).await?;
    let reservations: Result<Vec<_>, _> = rows
        .into_iter()
        .map(|r| DbTicketReservation::try_from(r))
        .collect();
    reservations
}

pub async fn db_select_one(db_client: &Client) -> Result<u64, tokio_postgres::Error> {
    db_client.execute("SELECT 1", &[]).await
}

pub fn sql_timestamp(sec_to_add: Option<i64>) -> NaiveDateTime {
    let timestamp: i64 = Utc::now()
        .checked_add_signed(Duration::seconds(sec_to_add.unwrap_or_default()))
        .and_then(|r| Some(r.timestamp_millis()))
        .expect("bad current timestamp");

    let created_at =
        NaiveDateTime::from_timestamp_opt(timestamp / 1000, (timestamp % 1000) as u32 * 1_000_000)
            .expect("bad native data time conversion");
    created_at
}

/// A generic query with parameters
pub struct Query<'a> {
    pub statement: Cow<'a, str>,
    pub params: Vec<&'a (dyn ToSql + Sync)>,
}

/// Create a new query
#[must_use]
pub fn query<'a>(statement: impl Into<Cow<'a, str>>) -> Query<'a> {
    Query::new(statement.into())
}

impl<'a> Query<'a> {
    /// creates a new query
    pub fn new(statement: Cow<'a, str>) -> Self {
        Self {
            statement,
            params: Vec::new(),
        }
    }

    /// Bind an unnamed parameter
    pub fn bind<T: Into<&'a (dyn ToSql + Sync)>>(mut self, value: T) -> Self {
        self.params.push(value.into());
        self
    }

    /// Binds multiple unnamed parameters
    pub fn bind_all<T: Into<&'a (dyn ToSql + Sync)>>(
        mut self,
        value: impl IntoIterator<Item = T>,
    ) -> Self {
        self.params.extend(value.into_iter().map(Into::into));
        self
    }

    /// allows us to query one row only
    pub async fn query_one<T>(
        self,
        db: &Client,
    ) -> Result<tokio_postgres::Row, tokio_postgres::Error>
    where
        T: Send + Unpin + 'static,
    {
        db.query_one(&self.statement.to_string(), &self.params)
            .await
    }
}
