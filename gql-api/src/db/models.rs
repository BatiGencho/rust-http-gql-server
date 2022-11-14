use crate::{
    auth::{Role, UserStatus},
    gql::models::{EventStatus, NewTicket},
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use slugify::slugify;
use std::convert::TryFrom;
use uuid::Uuid;

use super::sql::sql_timestamp;

// ------------USERS----------------
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbUser {
    pub id: uuid::Uuid,
    pub name: Option<String>,
    pub username: String,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub encrypted_secret_key: Option<String>,
    pub created_at: NaiveDateTime,
    pub wallet_id: String,
    pub wallet_balance: String,
    pub user_type: Role,
    pub user_status: UserStatus,
}

impl DbUser {
    pub fn new(
        id: Uuid,
        name: Option<String>,
        username: String,
        phone_number: Option<String>,
        email: Option<String>,
        password: Option<String>,
        encrypted_secret_key: Option<String>,
        user_type: Role,
        wallet_id: String,
        wallet_balance: String,
        user_status: UserStatus,
    ) -> Self {
        DbUser {
            id,
            name,
            username,
            phone_number,
            email,
            password,
            encrypted_secret_key,
            created_at: sql_timestamp(None),
            user_type,
            wallet_id,
            wallet_balance,
            user_status,
        }
    }
}

impl TryFrom<tokio_postgres::row::Row> for DbUser {
    type Error = tokio_postgres::Error;

    fn try_from(row: tokio_postgres::row::Row) -> Result<Self, Self::Error> {
        let created_at: NaiveDateTime = row.try_get(7)?;

        // user role
        let user_role: i16 = row.try_get(10)?;
        let user_role = Role::try_from(user_role).expect("must be a valid role");

        // user status
        let user_status: i16 = row.try_get(11)?;
        let user_status = UserStatus::try_from(user_status).expect("must be a valid user status");

        let user = DbUser {
            id: row.try_get(0)?,
            name: row.try_get(1).ok(),
            username: row.try_get(2)?,
            phone_number: row.try_get(3).ok(),
            email: row.try_get(4).ok(),
            password: row.try_get(5).ok(),
            encrypted_secret_key: row.try_get(6).ok(),
            created_at,
            wallet_id: row.try_get(8)?,
            wallet_balance: row.try_get(9)?,
            user_type: user_role,
            user_status,
        };
        Ok(user)
    }
}
// ------------EVENTS----------------
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbEvent {
    pub id: uuid::Uuid,
    pub event_name: String,
    pub event_slug: String,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
    pub entry_time: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub description: Option<String>,
    pub is_virtual: Option<bool>,
    pub is_featured: Option<bool>,
    pub venue_name: Option<String>,
    pub venue_location: Option<String>,
    pub cover_photo_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub event_status: EventStatus,
    pub created_by_user: uuid::Uuid,
}

impl DbEvent {
    pub fn new(event_name: &str, created_by_user: uuid::Uuid) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            event_slug: slugify!(&event_name, separator = "-"),
            event_name: event_name.to_owned(),
            start_date: None,
            end_date: None,
            entry_time: None,
            created_at: sql_timestamp(None),
            description: None,
            is_virtual: None,
            is_featured: None,
            venue_name: None,
            venue_location: None,
            cover_photo_url: None,
            thumbnail_url: None,
            event_status: EventStatus::Draft,
            created_by_user,
        }
    }
}

impl TryFrom<tokio_postgres::row::Row> for DbEvent {
    type Error = tokio_postgres::Error;

    fn try_from(row: tokio_postgres::row::Row) -> Result<Self, Self::Error> {
        let start_date: Option<NaiveDateTime> = row.try_get(3).ok();
        let end_date: Option<NaiveDateTime> = row.try_get(4).ok();
        let entry_time: Option<NaiveDateTime> = row.try_get(5).ok();
        let created_at: NaiveDateTime = row.try_get(6)?;

        let event_status: i16 = row.try_get(14)?;
        let event_status =
            EventStatus::try_from(event_status).expect("must be a valid event status");

        Ok(DbEvent {
            id: row.try_get(0)?,
            event_name: row.try_get(1)?,
            event_slug: row.try_get(2)?,
            start_date,
            end_date,
            entry_time,
            created_at,
            description: row.try_get(7).ok(),
            is_virtual: row.try_get(8).ok(),
            is_featured: row.try_get(9).ok(),
            venue_name: row.try_get(10).ok(),
            venue_location: row.try_get(11).ok(),
            cover_photo_url: row.try_get(12).ok(),
            thumbnail_url: row.try_get(13).ok(),
            event_status,
            created_by_user: row.try_get(15)?,
        })
    }
}
// -------------TICKETS----------------
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbTicket {
    pub id: uuid::Uuid,
    pub created_at: NaiveDateTime,
    pub ticket_name: String,
    pub ticket_slug: String,
    pub description: Option<String>,
    pub price: Option<String>,
    pub max_release_price: Option<String>,
    pub quantity_available: Option<i32>,
    pub min_purchase_quantity: Option<i32>,
    pub max_purchase_quantity: Option<i32>,
    pub allow_transfers: Option<bool>,
    pub event_id: uuid::Uuid,
}

impl DbTicket {
    pub fn new(ticket: NewTicket, db_event: &DbEvent) -> Self {
        let ticket_slug = format!(
            "{}-{}",
            &db_event.event_slug,
            slugify!(&ticket.ticket_name, separator = "-")
        );
        Self {
            id: uuid::Uuid::new_v4(),
            ticket_name: ticket.ticket_name,
            ticket_slug,
            created_at: sql_timestamp(None),
            description: ticket.description,
            price: ticket.price,
            max_release_price: ticket.max_release_price,
            quantity_available: ticket.quantity_available,
            min_purchase_quantity: ticket.min_purchase_quantity,
            max_purchase_quantity: ticket.max_purchase_quantity,
            allow_transfers: ticket.allow_transfers,
            event_id: db_event.id,
        }
    }
}

impl TryFrom<tokio_postgres::row::Row> for DbTicket {
    type Error = tokio_postgres::Error;

    fn try_from(row: tokio_postgres::row::Row) -> Result<Self, Self::Error> {
        let created_at: NaiveDateTime = row.try_get(1)?;

        Ok(DbTicket {
            id: row.try_get(0)?,
            created_at,
            ticket_name: row.try_get(2)?,
            ticket_slug: row.try_get(3)?,
            description: row.try_get(4).ok(),
            price: row.try_get(5).ok(),
            max_release_price: row.try_get(6).ok(),
            quantity_available: row.try_get(7).ok(),
            min_purchase_quantity: row.try_get(8).ok(),
            max_purchase_quantity: row.try_get(9).ok(),
            allow_transfers: row.try_get(10).ok(),
            event_id: row.try_get(11)?,
        })
    }
}

// -------------SELLER LOGIN SESSIONS---------------
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbSession {
    pub id: uuid::Uuid,
    pub expires_at: NaiveDateTime,
    pub login_code: String,
    pub is_used: bool,
    pub user_id: Option<uuid::Uuid>,
}

impl DbSession {
    pub fn new(
        id: uuid::Uuid,
        expires_at: NaiveDateTime,
        login_code: String,
        is_used: bool,
        user_id: Option<uuid::Uuid>,
    ) -> Self {
        DbSession {
            id,
            expires_at,
            login_code,
            is_used,
            user_id,
        }
    }
}

impl TryFrom<tokio_postgres::row::Row> for DbSession {
    type Error = tokio_postgres::Error;

    fn try_from(row: tokio_postgres::row::Row) -> Result<Self, Self::Error> {
        let expires_at: NaiveDateTime = row.try_get(1)?;
        Ok(DbSession {
            id: row.try_get(0)?,
            expires_at,
            login_code: row.try_get(2)?,
            is_used: row.try_get(3)?,
            user_id: row.try_get(4).ok(),
        })
    }
}

// -------------BUYER SIGNUP SESSIONS---------------
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbBuyerSignupSession {
    pub id: uuid::Uuid,
    pub created_at: NaiveDateTime,
    pub verification_code: String,
    pub phone_number: String,
    pub is_verified: bool,
}

impl DbBuyerSignupSession {
    pub fn new(
        id: uuid::Uuid,
        created_at: NaiveDateTime,
        verification_code: String,
        phone_number: String,
        is_verified: bool,
    ) -> Self {
        DbBuyerSignupSession {
            id,
            created_at,
            verification_code,
            phone_number,
            is_verified,
        }
    }
}

impl TryFrom<tokio_postgres::row::Row> for DbBuyerSignupSession {
    type Error = tokio_postgres::Error;

    fn try_from(row: tokio_postgres::row::Row) -> Result<Self, Self::Error> {
        let created_at: NaiveDateTime = row.try_get(1)?;
        Ok(DbBuyerSignupSession {
            id: row.try_get(0)?,
            created_at,
            verification_code: row.try_get(2)?,
            phone_number: row.try_get(3)?,
            is_verified: row.try_get(4)?,
        })
    }
}

// -------------BUYER RECOVERY SESSIONS---------------
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbBuyerRecoverySession {
    pub id: uuid::Uuid,
    pub created_at: NaiveDateTime,
    pub recovery_code: String,
    pub phone_number: String,
    pub is_recovered: bool,
    pub created_by_user: uuid::Uuid,
}

impl DbBuyerRecoverySession {
    pub fn new(
        id: uuid::Uuid,
        created_at: NaiveDateTime,
        recovery_code: String,
        phone_number: String,
        is_recovered: bool,
        created_by_user: uuid::Uuid,
    ) -> Self {
        DbBuyerRecoverySession {
            id,
            created_at,
            recovery_code,
            phone_number,
            is_recovered,
            created_by_user,
        }
    }
}

impl TryFrom<tokio_postgres::row::Row> for DbBuyerRecoverySession {
    type Error = tokio_postgres::Error;

    fn try_from(row: tokio_postgres::row::Row) -> Result<Self, Self::Error> {
        let created_at: NaiveDateTime = row.try_get(1)?;
        Ok(DbBuyerRecoverySession {
            id: row.try_get(0)?,
            created_at,
            recovery_code: row.try_get(2)?,
            phone_number: row.try_get(3)?,
            is_recovered: row.try_get(4)?,
            created_by_user: row.try_get(5)?,
        })
    }
}

// ------------TICKET RESERVATIONS----------------
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbTicketReservation {
    pub id: uuid::Uuid,
    pub created_at: NaiveDateTime,
    pub verification_code: String,
    pub event_id: uuid::Uuid,
    pub ticket_id: uuid::Uuid,
    pub user_id: uuid::Uuid,
}

impl DbTicketReservation {
    pub fn new(
        id: uuid::Uuid,
        created_at: NaiveDateTime,
        verification_code: &str,
        event_id: uuid::Uuid,
        ticket_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Self {
        DbTicketReservation {
            id,
            created_at,
            verification_code: verification_code.to_owned(),
            event_id,
            ticket_id,
            user_id,
        }
    }
}

impl TryFrom<tokio_postgres::row::Row> for DbTicketReservation {
    type Error = tokio_postgres::Error;

    fn try_from(row: tokio_postgres::row::Row) -> Result<Self, Self::Error> {
        let created_at: NaiveDateTime = row.try_get(1)?;
        Ok(DbTicketReservation {
            id: row.try_get(0)?,
            created_at,
            verification_code: row.try_get(2)?,
            event_id: row.try_get(3)?,
            ticket_id: row.try_get(4)?,
            user_id: row.try_get(5)?,
        })
    }
}
// -----------S3 FILES-----------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AssetFile {
    pub id: uuid::Uuid,
    pub s3_bucket: String,
    pub s3_absolute_key: String,
    pub ipfs_hash: Option<String>,
    pub event_id: uuid::Uuid,
}

impl AssetFile {
    pub fn new(
        s3_bucket: impl Into<String>,
        s3_absolute_key: impl Into<String>,
        ipfs_hash: Option<String>,
        event_id: uuid::Uuid,
    ) -> Self {
        Self::new_with_id(
            Uuid::new_v4(),
            s3_bucket,
            s3_absolute_key,
            ipfs_hash,
            event_id,
        )
    }

    pub fn new_with_id(
        id: Uuid,
        s3_bucket: impl Into<String>,
        s3_absolute_key: impl Into<String>,
        ipfs_hash: Option<String>,
        event_id: uuid::Uuid,
    ) -> Self {
        Self {
            id,
            s3_bucket: s3_bucket.into(),
            s3_absolute_key: s3_absolute_key.into(),
            ipfs_hash,
            event_id,
        }
    }
}

impl TryFrom<tokio_postgres::row::Row> for AssetFile {
    type Error = tokio_postgres::Error;

    fn try_from(value: tokio_postgres::row::Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.try_get(0)?,
            s3_bucket: value.try_get(1)?,
            s3_absolute_key: value.try_get(2)?,
            ipfs_hash: value.try_get(3)?,
            event_id: value.try_get(4)?,
        })
    }
}
