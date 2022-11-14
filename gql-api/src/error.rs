use crate::http::models::{ErrorResponse, FieldError};
use displaydoc::Display as DisplayDoc;
use near_account_id::ParseAccountError;
use pusher_client::error::PusherError;
use reqwest::StatusCode;
use std::{convert::Infallible, error::Error as StdError, net::AddrParseError};
use thiserror::Error;
use twilio_client::error::TwilioError;
use validator::{ValidationErrors, ValidationErrorsKind};
use warp::{Rejection, Reply};

/// Password hashing error types.
#[derive(Debug, DisplayDoc, Error)]
pub enum Error {
    /// Auth error: `{0}`
    Auth(AuthError),
    /// Hash error: `{0}`
    Hash(HashError),
    /// Postgres error: `{0}`
    Postgres(tokio_postgres::Error),
    /// Server parse address error: `{0}`
    ParseAddr(AddrParseError),
    /// Unparsable UUID error: `{0}`
    UnparsableUuid(String),
    /// Missing certificate error
    MissingCertificate,
    /// User error: `{0}`
    User(UserError),
    /// Event error: `{0}`
    Event(EventError),
    /// Ticket error: `{0}`
    Ticket(TicketError),
    /// Request error: `{0}`
    Request(RequestError),
    /// Signature error: `{0}`
    Signature(ed25519_dalek::ed25519::Error),
    /// Base58 Encode/Decode error: `{0}`
    Base58(bs58::decode::Error),
    /// Grpc error: `{0}`
    Grpc(GrpcError),
    /// Session error: `{0}`
    Session(SessionError),
    /// Pusher error: `{0}`
    Pusher(PusherError),
    /// Twilio error: `{0}`
    Twilio(TwilioError),
}

impl warp::reject::Reject for Error {}

/// Password hashing error types.
#[derive(Debug, DisplayDoc, Error, PartialEq)]
pub enum HashError {
    /// Argon2 Encode error: `{0}`
    Encode(argon2::Error),
    /// Argon2 Verify error: `{0}`
    Verify(argon2::Error),
}

impl warp::reject::Reject for HashError {}

/// Auth errors
#[derive(Clone, Debug, DisplayDoc, Error, PartialEq)]
pub enum AuthError {
    /// Wrong Credentials
    WrongCredentialsError,
    /// JWT Token not valid
    JWTTokenError,
    /// JWT Token Creation Error
    JWTTokenCreationError,
    /// No Auth Header
    NoAuthHeaderError,
    /// Invalid Auth Header
    InvalidAuthHeaderError,
    /// No Permission
    NoPermissionError,
    /// Bad Encoded User Role: `{0}`
    BadEncodedUserRole(String),
}

impl warp::reject::Reject for AuthError {}

/// User-related errors
#[derive(Clone, Debug, DisplayDoc, Error, PartialEq)]
pub enum UserError {
    /// User not found
    UserNotFound,
    /// User has no password
    NoPassword,
    /// Unknown User Role: `{0}`
    UnknownUserRole(String),
    /// Unknown User Status: `{0}`
    UnknownUserStatus(String),
    /// Unallowed User Role: `{0}`
    UnallowedUserRole(String),
    /// Only sellers allowed/User is not a seller
    OnlySeller,
    /// Only buyers allowed/User is not a buyer
    OnlyBuyer,
    /// Unimplemented use case. Check parameters submitted
    UnimplementedCase,
    /// Wrong wallet public key
    WrongWalletPubKey,
    /// Unparsable implicit account
    AccountParse(ParseAccountError),
    /// Bad implicit account
    BadImplicitAccount,
    /// Bad normal account
    BadNormalAccount,
    /// Missing signature
    MissingSignature,
    /// Missing account/wallet Id
    MissingWalletId,
    /// Missing password
    MissingPassword,
    /// Missing pubic key
    MissingPubKey,
    /// User wallet creating Failed
    WalletCreationFailed,
    /// Bad signature
    BadSignature,
    /// Unavailable Username
    UnavailableUsername,
    /// Unavailable Name
    UnavailableName,
    /// Unavailable Email
    UnavailableEmail,
    /// Unavailable Phone Number
    UnavailablePhoneNumber,
    /// User is not verified
    UnverifiedUser,
}

impl warp::reject::Reject for UserError {}

/// Event-related errors
#[derive(Clone, Debug, DisplayDoc, Error, PartialEq)]
pub enum EventError {
    /// Non-existing event with uuid: `{0}`
    NoExistEventUuid(String),
}

impl warp::reject::Reject for EventError {}

/// Ticket-related errors
#[derive(Clone, Debug, DisplayDoc, Error, PartialEq)]
pub enum TicketError {
    /// Mismatching ticket event id: `{0}`
    TicketEventMismatch(String),
    /// Non-existing ticket with uuid: `{0}`
    NoExistTicketUuid(String),
    /// Non-existing ticket with code: `{0}`
    NoExistTicketWithCode(String),
    /// Wrong ticket user reserved: `{0}`
    WrongUserReserved(String),
    /// No ticket reservations found for code: `{0}`
    NoTicketReservationsForCode(String),
    /// Ticket has already been reserved for the user: `{0}`
    AlreadyReservedForUser(String),
}

impl warp::reject::Reject for TicketError {}

/// Request-related errors
#[derive(Clone, Debug, DisplayDoc, Error, PartialEq)]
pub enum RequestError {
    /// JSON path error: `{0}`
    JSONPathError(String),
    /// validation error: `{0}`
    ValidationError(ValidationErrors),
}

impl warp::reject::Reject for RequestError {}

/// Request-related errors
#[derive(Clone, Debug, DisplayDoc, Error, PartialEq)]
pub enum SessionError {
    /// Verification code for the session is incorrect: `{0}`
    SessionVerificationCodeMismatch(String),
    /// Recovery code for the session is incorrect: `{0}`
    SessionRecoveryCodeMismatch(String),
    /// No session found for uuid: `{0}`
    SessionNotFoundForUuid(String),
    /// No session for token: `{0}`
    NoSessionForToken(String),
    /// Used session for token: `{0}`
    UsedSession(String),
    /// Expired session for token: `{0}`
    ExpiredSession(String),
}

impl warp::reject::Reject for SessionError {}

/// grpc-related errors
#[derive(Debug, DisplayDoc, Error)]
pub enum GrpcError {
    /// tonic transport error: `{0}`
    Transport(tonic::transport::Error),
    /// tonic call error: `{0}`
    Call(tonic::Status),
}

impl warp::reject::Reject for GrpcError {}

pub async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let (code, message, errors) = if err.is_not_found() {
        eprintln!("NOT FOUND error");
        (StatusCode::NOT_FOUND, "Not Found".to_string(), None)
    } else if let Some(e) = err.find::<warp::filters::body::BodyDeserializeError>() {
        eprintln!("Invalid body error");
        (
            StatusCode::BAD_REQUEST,
            e.source()
                .map(|cause| cause.to_string())
                .unwrap_or_else(|| "BAD_REQUEST".to_string()),
            None,
        )
    } else if let Some(Error::Auth(e)) = err.find::<Error>() {
        match e {
            AuthError::WrongCredentialsError => (StatusCode::FORBIDDEN, e.to_string(), None),
            AuthError::NoPermissionError => (StatusCode::UNAUTHORIZED, e.to_string(), None),
            AuthError::JWTTokenError => (StatusCode::UNAUTHORIZED, e.to_string(), None),
            AuthError::BadEncodedUserRole(_) => (StatusCode::UNAUTHORIZED, e.to_string(), None),
            AuthError::JWTTokenCreationError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
                None,
            ),
            _ => (StatusCode::BAD_REQUEST, e.to_string(), None),
        }
    } else if let Some(Error::Signature(e)) = err.find::<Error>() {
        eprintln!("Invalid signature error");
        (StatusCode::UNAUTHORIZED, e.to_string(), None)
    } else if let Some(Error::Session(e)) = err.find::<Error>() {
        eprintln!("Session error");
        (StatusCode::FORBIDDEN, e.to_string(), None)
    } else if let Some(Error::Base58(e)) = err.find::<Error>() {
        eprintln!("Invalid base58 error");
        (StatusCode::BAD_REQUEST, e.to_string(), None)
    } else if let Some(Error::UnparsableUuid(e)) = err.find::<Error>() {
        eprintln!("Unparsable uuid error");
        (StatusCode::BAD_REQUEST, e.to_string(), None)
    } else if let Some(Error::Request(e)) = err.find::<Error>() {
        eprintln!("request error: {:?}", e.to_string());
        match e {
            RequestError::JSONPathError(_) => (StatusCode::BAD_REQUEST, e.to_string(), None),
            RequestError::ValidationError(val_errs) => {
                let errors: Vec<FieldError> = val_errs
                    .errors()
                    .iter()
                    .map(|error_kind| FieldError {
                        field: error_kind.0.to_string(),
                        field_errors: match error_kind.1 {
                            ValidationErrorsKind::Struct(struct_err) => {
                                validation_errs_to_str_vec(struct_err)
                            }
                            ValidationErrorsKind::Field(field_errs) => field_errs
                                .iter()
                                .map(|fe| format!("{}: {:?}", fe.code, fe.params))
                                .collect(),
                            ValidationErrorsKind::List(vec_errs) => vec_errs
                                .iter()
                                .map(|ve| {
                                    format!(
                                        "{}: {:?}",
                                        ve.0,
                                        validation_errs_to_str_vec(ve.1).join(" | "),
                                    )
                                })
                                .collect(),
                        },
                    })
                    .collect();
                (
                    StatusCode::BAD_REQUEST,
                    "field errors".to_string(),
                    Some(errors),
                )
            }
        }
    } else if let Some(Error::Postgres(e)) = err.find::<Error>() {
        eprintln!("postgres error: {:?}", e.to_string());
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
            None,
        )
    } else if let Some(Error::Grpc(e)) = err.find::<Error>() {
        eprintln!("grpc error: {:?}", e.to_string());
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
            None,
        )
    } else if let Some(Error::Pusher(e)) = err.find::<Error>() {
        eprintln!("grpc error: {:?}", e.to_string());
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
            None,
        )
    } else if let Some(Error::Twilio(e)) = err.find::<Error>() {
        eprintln!("twilio error: {:?}", e.to_string());
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
            None,
        )
    } else if let Some(Error::Hash(e)) = err.find::<Error>() {
        eprintln!("hashing error: {:?}", e.to_string());
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
            None,
        )
    } else if let Some(Error::User(e)) = err.find::<Error>() {
        eprintln!("user error: {:?}", e.to_string());
        (StatusCode::FORBIDDEN, e.to_string(), None)
    } else if let Some(Error::Event(e)) = err.find::<Error>() {
        eprintln!("event error: {:?}", e.to_string());
        (StatusCode::FORBIDDEN, e.to_string(), None)
    } else if let Some(Error::Ticket(e)) = err.find::<Error>() {
        eprintln!("ticket error: {:?}", e.to_string());
        (StatusCode::FORBIDDEN, e.to_string(), None)
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        eprintln!("MethodNotAllowed error");
        (
            StatusCode::METHOD_NOT_ALLOWED,
            "Method Not Allowed".to_string(),
            None,
        )
    } else {
        eprintln!("any other unhandled error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
            None,
        )
    };

    let json = warp::reply::json(&ErrorResponse {
        status: code.to_string(),
        message: message.into(),
        errors: errors,
    });

    Ok(warp::reply::with_status(json, code))
}

fn validation_errs_to_str_vec(ve: &ValidationErrors) -> Vec<String> {
    ve.field_errors()
        .iter()
        .map(|fe| {
            format!(
                "{}: errors: {}",
                fe.0,
                fe.1.iter()
                    .map(|ve| format!("{}: {:?}", ve.code, ve.params))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        })
        .collect()
}
