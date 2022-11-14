use crate::db::models::{DbBuyerRecoverySession, DbBuyerSignupSession, DbEvent, DbTicket, DbUser};
use serde::{Deserialize, Serialize};
use std::convert::From;
use validator::Validate;

// -------------BUYER CREATE RECOVERY CODE------------------

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct BuyerCreateRecoveryCodeRequest {
    #[validate(phone)]
    pub phone_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct BuyerCreateRecoveryCodeResponse {
    pub session_id: String,
}

impl From<DbBuyerRecoverySession> for BuyerCreateRecoveryCodeResponse {
    fn from(db_buyer_recovery_code_session: DbBuyerRecoverySession) -> Self {
        BuyerCreateRecoveryCodeResponse {
            session_id: db_buyer_recovery_code_session.id.to_string(),
        }
    }
}

// -------------BUYER VERIFY RECOVERY CODE------------------

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct BuyerVerifyRecoveryCodeRequest {
    pub session_id: String,
    #[validate(length(equal = 6))]
    pub recovery_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct BuyerVerifyRecoveryCodeResponse {
    pub encrypted_secret_key: String,
    pub jwt: Option<String>,
    pub wallet_id: String,
}

impl From<DbUser> for BuyerVerifyRecoveryCodeResponse {
    fn from(db_user: DbUser) -> Self {
        BuyerVerifyRecoveryCodeResponse {
            encrypted_secret_key: db_user.encrypted_secret_key.unwrap_or_default(), // should always be Some
            jwt: None,
            wallet_id: db_user.wallet_id,
        }
    }
}

// -------------BUYER REGISTER PHONE------------------

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct BuyerRegisterPhoneRequest {
    #[validate(phone)]
    pub phone_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct BuyerRegisterPhoneResponse {
    pub session_id: String,
}

impl From<DbBuyerSignupSession> for BuyerRegisterPhoneResponse {
    fn from(db_buyer_signup_session: DbBuyerSignupSession) -> Self {
        BuyerRegisterPhoneResponse {
            session_id: db_buyer_signup_session.id.to_string(),
        }
    }
}

// -----------BUYER VERIFY PHONE--------------------

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct BuyerVerifyPhoneRequest {
    pub session_id: String,
    #[validate(length(equal = 6))]
    pub verification_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct BuyerVerifyPhoneResponse {
    pub is_verified: bool,
}

impl From<DbBuyerSignupSession> for BuyerVerifyPhoneResponse {
    fn from(db_buyer_signup_session: DbBuyerSignupSession) -> Self {
        BuyerVerifyPhoneResponse {
            is_verified: db_buyer_signup_session.is_verified,
        }
    }
}

// -----------BUYER SIGNUP--------------------

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct BuyerSignupRequest {
    #[validate(length(min = 2, max = 20))]
    pub name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 2, max = 20))]
    pub username: String,
    pub password: Option<String>,
    #[validate(length(min = 4, max = 32))]
    pub secret: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuyerSignupResponse {
    pub id: String,
    pub name: Option<String>,
    pub username: String,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub created_at: i64,
    pub wallet_id: String,
    pub wallet_pub_key: Option<String>,
    pub wallet_encrypted_secret_key: Option<String>,
    pub wallet_balance: String,
    pub user_type: String,
    pub user_status: String,
    pub jwt: Option<String>,
}

impl From<DbUser> for BuyerSignupResponse {
    fn from(db_user: DbUser) -> Self {
        BuyerSignupResponse {
            id: db_user.id.to_string(),
            name: db_user.name,
            username: db_user.username,
            phone_number: db_user.phone_number,
            email: db_user.email,
            created_at: db_user.created_at.timestamp(),
            wallet_id: db_user.wallet_id,
            wallet_pub_key: None,
            wallet_encrypted_secret_key: db_user.encrypted_secret_key,
            wallet_balance: db_user.wallet_balance,
            user_type: db_user.user_type.to_string(),
            user_status: db_user.user_status.to_string(),
            jwt: None,
        }
    }
}

// ---------------------------

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SigninRequest {
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 5, max = 50))]
    pub name: Option<String>,
    #[validate(length(min = 5, max = 50))]
    pub username: String,
    #[validate(phone)]
    pub phone_number: Option<String>,
    #[validate(length(min = 5, max = 50))]
    pub password: Option<String>,
    pub signature: Option<String>,
    #[validate(length(min = 5, max = 64))]
    pub wallet_id: Option<String>,
    #[validate(length(min = 40, max = 45))]
    pub pub_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SigninWithPasswordRequest {
    #[validate(length(min = 5, max = 64))]
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SigninResponse {
    pub token: String,
}

// ---------------------------

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CheckUsernameRequest {
    #[validate(length(min = 2, max = 20))]
    pub username: String,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckUsernameResponse {
    pub available: bool,
}

// ---------------------------

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateLoginCodeRequest {}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateLoginCodeResponse {
    pub code: String,
    pub expires_at: i64,
}

// ---------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct VerifyLoginCodeRequest {
    pub code: String,
    pub signature: String,
    #[validate(length(min = 5, max = 64))]
    pub wallet_id: String,
    #[validate(length(min = 40, max = 45))]
    pub pub_key: String,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct VerifyLoginCodeResponse {}

// ---------------------------

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct EventTicketReservation {
    pub ticket_id: String,
    pub quantity: i64,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct EventTicketGetVerificationCodeRequest {
    pub event_id: String,
    pub reservations: Vec<EventTicketReservation>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct EventGetVerificationCodeResponse {
    pub verification_code: String,
}

// ---------------------------
#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GetEventFromVerificationCodeRequest {
    pub verification_code: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GetEventFromVerificationCodeResponse {
    pub id: String,
    pub event_name: String,
    pub event_slug: String,
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
    pub entry_time: Option<i64>,
    pub created_at: i64,
    pub is_virtual: Option<bool>,
    pub is_featured: Option<bool>,
    pub venue_name: Option<String>,
    pub venue_location: Option<String>,
    pub cover_photo_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub event_status: String,
    pub tickets: Vec<Ticket>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Ticket {
    pub id: String,
    pub created_at: i64,
    pub ticket_name: String,
    pub ticket_slug: String,
    pub description: Option<String>,
    pub price: Option<String>,
    pub max_release_price: Option<String>,
    pub quantity_available: Option<i32>,
    pub min_purchase_quantity: Option<i32>,
    pub max_purchase_quantity: Option<i32>,
    pub allow_transfers: Option<bool>,
    pub event_id: String,
}

impl From<DbTicket> for Ticket {
    fn from(ticket: DbTicket) -> Self {
        Ticket {
            id: ticket.id.to_string(),
            created_at: ticket.created_at.timestamp_millis(),
            ticket_name: ticket.ticket_name,
            ticket_slug: ticket.ticket_slug,
            description: ticket.description,
            price: ticket.price,
            max_release_price: ticket.max_release_price,
            quantity_available: ticket.quantity_available,
            min_purchase_quantity: ticket.min_purchase_quantity,
            max_purchase_quantity: ticket.max_purchase_quantity,
            allow_transfers: ticket.allow_transfers,
            event_id: ticket.event_id.to_string(),
        }
    }
}

impl GetEventFromVerificationCodeResponse {
    pub fn new(db_event: DbEvent, tickets: Vec<DbTicket>) -> Self {
        GetEventFromVerificationCodeResponse {
            id: db_event.id.to_string(),
            event_name: db_event.event_name,
            event_slug: db_event.event_slug,
            start_date: db_event.start_date.map(|date| date.timestamp_millis()),
            end_date: db_event.end_date.map(|date| date.timestamp_millis()),
            entry_time: db_event.entry_time.map(|date| date.timestamp_millis()),
            created_at: db_event.created_at.timestamp_millis(),
            is_virtual: db_event.is_virtual,
            is_featured: db_event.is_featured,
            venue_name: db_event.venue_name,
            venue_location: db_event.venue_location,
            cover_photo_url: db_event.cover_photo_url,
            thumbnail_url: db_event.thumbnail_url,
            event_status: db_event.event_status.to_string(),
            tickets: tickets.into_iter().map(Ticket::from).collect(),
        }
    }
}

// ---------------------------
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub message: String,
    pub status: String,
    pub errors: Option<Vec<FieldError>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldError {
    pub field: String,
    pub field_errors: Vec<String>,
}
