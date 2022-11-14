use crate::error::{AuthError, Error, UserError};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use uuid::Uuid;
use warp::{reject, Rejection};

const BEARER: &str = "Bearer ";
const JWT_SECRET: &[u8] = b"secret";

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: String,
    role: String,
    exp: usize,
}

/// A user role
#[repr(i16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Admin = 0,
    Seller = 1,
    Buyer = 2,
    SuperAdmin = 4,
}

impl From<Role> for i16 {
    fn from(role: Role) -> i16 {
        role as i16
    }
}

impl TryFrom<i16> for Role {
    type Error = Error;

    fn try_from(n: i16) -> Result<Self, Self::Error> {
        match n {
            0 => Ok(Role::Admin),
            1 => Ok(Role::Seller),
            2 => Ok(Role::Buyer),
            4 => Ok(Role::SuperAdmin),
            _ => Err(Error::User(UserError::UnknownUserRole(n.to_string()))),
        }
    }
}

/// Maps a string to a Role
impl TryFrom<&str> for Role {
    type Error = Error;

    fn try_from(role: &str) -> Result<Self, Self::Error> {
        match role.to_lowercase().as_str() {
            "buyer" => Ok(Role::Buyer),
            "seller" => Ok(Role::Seller),
            "admin" => Ok(Role::Admin),
            "superadmin" => Ok(Role::SuperAdmin),
            _ => Err(Error::User(UserError::UnknownUserRole(role.to_string()))),
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Buyer => write!(f, "buyer"),
            Role::Seller => write!(f, "seller"),
            Role::Admin => write!(f, "admin"),
            Role::SuperAdmin => write!(f, "superadmin"),
        }
    }
}

/// A user status
#[repr(i16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    Unverified = 0,
    PhoneVerified = 1,
}

impl From<UserStatus> for i16 {
    fn from(user_status: UserStatus) -> i16 {
        user_status as i16
    }
}

impl TryFrom<i16> for UserStatus {
    type Error = Error;

    fn try_from(n: i16) -> Result<Self, Self::Error> {
        match n {
            0 => Ok(UserStatus::Unverified),
            1 => Ok(UserStatus::PhoneVerified),
            _ => Err(Error::User(UserError::UnknownUserStatus(n.to_string()))),
        }
    }
}

/// Maps a string to a status
impl TryFrom<&str> for UserStatus {
    type Error = Error;

    fn try_from(user_status: &str) -> Result<Self, Self::Error> {
        match user_status.to_lowercase().as_str() {
            "unverified" => Ok(UserStatus::Unverified),
            "phone_verified" => Ok(UserStatus::PhoneVerified),
            _ => Err(Error::User(UserError::UnknownUserStatus(
                user_status.to_string(),
            ))),
        }
    }
}

impl fmt::Display for UserStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserStatus::Unverified => write!(f, "unverified"),
            UserStatus::PhoneVerified => write!(f, "phone_verified"),
        }
    }
}

pub fn create_jwt(uid: &str, role: &Role) -> Result<String, AuthError> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::minutes(60))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: uid.to_owned(),
        role: role.to_string(),
        exp: expiration as usize,
    };
    let header = Header::new(Algorithm::HS512);
    encode(&header, &claims, &EncodingKey::from_secret(JWT_SECRET))
        .map_err(|_| AuthError::JWTTokenCreationError)
}

fn jwt_from_header(headers: &HeaderMap<HeaderValue>) -> Result<String, AuthError> {
    let header = match headers.get(AUTHORIZATION) {
        Some(v) => v,
        None => return Err(AuthError::NoAuthHeaderError),
    };
    let auth_header = match std::str::from_utf8(header.as_bytes()) {
        Ok(v) => v,
        Err(_) => return Err(AuthError::NoAuthHeaderError),
    };
    if !auth_header.starts_with(BEARER) {
        return Err(AuthError::InvalidAuthHeaderError);
    }
    Ok(auth_header.trim_start_matches(BEARER).to_owned())
}

pub async fn authorize(
    (roles, headers): (Vec<Role>, HeaderMap<HeaderValue>),
) -> Result<Uuid, Rejection> {
    match jwt_from_header(&headers) {
        Ok(jwt) => {
            let decoded = decode::<Claims>(
                &jwt,
                &DecodingKey::from_secret(JWT_SECRET),
                &Validation::new(Algorithm::HS512),
            )
            .map_err(|_| reject::custom(Error::Auth(AuthError::JWTTokenError)))?;

            let token_role = Role::try_from(decoded.claims.role.as_str()).map_err(|_| {
                reject::custom(Error::Auth(AuthError::BadEncodedUserRole(
                    decoded.claims.role,
                )))
            })?;
            let token_role = roles.iter().find(|&role| role.eq(&token_role));
            if token_role.is_none() {
                return Err(reject::custom(Error::Auth(AuthError::NoPermissionError)));
            }

            let user_id = Uuid::parse_str(&decoded.claims.sub).map_err(|_| {
                reject::custom(Error::UnparsableUuid(decoded.claims.sub.to_string()))
            })?;
            Ok(user_id)
        }
        Err(e) => return Err(reject::custom(Error::Auth(e))),
    }
}
