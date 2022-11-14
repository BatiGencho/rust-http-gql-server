use super::models::{
    BuyerCreateRecoveryCodeRequest, BuyerCreateRecoveryCodeResponse, BuyerRegisterPhoneRequest,
    BuyerRegisterPhoneResponse, BuyerSignupRequest, BuyerSignupResponse, BuyerVerifyPhoneRequest,
    BuyerVerifyPhoneResponse, BuyerVerifyRecoveryCodeRequest, BuyerVerifyRecoveryCodeResponse,
    CheckUsernameRequest, CheckUsernameResponse, CreateLoginCodeRequest, CreateLoginCodeResponse,
    EventGetVerificationCodeResponse, EventTicketGetVerificationCodeRequest,
    GetEventFromVerificationCodeRequest, GetEventFromVerificationCodeResponse, SigninRequest,
    SigninResponse, SigninWithPasswordRequest, VerifyLoginCodeRequest, VerifyLoginCodeResponse,
};
use crate::{
    auth::{create_jwt, Role, UserStatus},
    db::{
        models::{
            DbBuyerRecoverySession, DbBuyerSignupSession, DbSession, DbTicketReservation, DbUser,
        },
        sql::{
            db_get_buyer_recovery_session_by_id, db_get_buyer_signup_session_by_id,
            db_get_event_by_id, db_get_session_by_login_code, db_get_ticket_by_id,
            db_get_ticket_reservations_by_code, db_get_ticket_reservations_by_user_id,
            db_get_tickets_by_event_id, db_get_user_by_email, db_get_user_by_id,
            db_get_user_by_name, db_get_user_by_phone_number, db_get_user_by_username,
            db_get_user_by_wallet_id, db_get_users_by_username, db_insert_buyer_recovery_session,
            db_insert_buyer_signup_session, db_insert_session, db_insert_ticket_reservation,
            db_insert_user, db_select_one, db_update_buyer_recovery_session,
            db_update_buyer_signup_session, db_update_session_info, sql_timestamp,
        },
    },
    error::{AuthError, Error, EventError, RequestError, SessionError, TicketError, UserError},
    gql::schema::Context as ResourcesContext,
    grpc::near_api::{
        AesEncryptDataResponse, CreateAccountResponse, GenerateImplicitAccountResponse, TxStatus,
    },
    security::crypto::check_normal_account,
    security::password::{hash_password, verify_password},
};
use bytes::buf::Buf;
use chrono::Utc;
use pusher_client::{channels::PusherChannels, events::PusherEvents};
use reqwest::StatusCode;
use std::convert::From;
use std::sync::Arc;
use twilio_client::models::SmsMessage;
use uuid::Uuid;
use validator::Validate;
use warp::{reject, Rejection};
use wasmium_random::WasmiumRandom;

// TODO: put these in a config file or secret
const MESSAGE: &'static str = "SECRET";
const WALLET_CREATION_DEPOSIT_AMOUNT: &'static str = "0.2"; // near
const NEAR_NETWORK_MODE: &'static str = "testnet";
const VERIFICATION_SMS_TEXT: &'static str = "Your verification code is: ";
const RECOVERY_SMS_TEXT: &'static str = "Your recovery code is: ";

// healthcheck route
pub async fn health(ctx: Arc<ResourcesContext>) -> Result<impl warp::Reply, Rejection> {
    db_select_one(&ctx.db_client)
        .await
        .map_err(|e| reject::custom(Error::Postgres(e)))?;
    Ok(StatusCode::OK)
}

// check if a given username exists
pub async fn check_username(
    ctx: Arc<ResourcesContext>,
    buf: impl Buf,
) -> Result<impl warp::Reply, Rejection> {
    let des = &mut serde_json::Deserializer::from_reader(buf.reader());
    let req_body: CheckUsernameRequest = serde_path_to_error::deserialize(des)
        .map_err(|e| reject::custom(Error::Request(RequestError::JSONPathError(e.to_string()))))?;

    let users = db_get_users_by_username(&ctx.db_client, &req_body.username)
        .await
        .map_err(Error::Postgres)?;
    let near_account_id = format!("{}.{}", req_body.username, NEAR_NETWORK_MODE);

    // check for available username
    let is_available = {
        let mut lock = ctx.grpc_near_client.lock().await;
        let is_available = lock
            .check_available_account_id(&near_account_id)
            .await
            .map_err(|e| reject::custom(Error::Grpc(e)))?
            .is_available;
        drop(lock);
        is_available
    };

    Ok(warp::reply::json(&CheckUsernameResponse {
        available: users.len() == 0 && is_available,
    }))
}

// seller signup/signin with wallet
pub async fn signin(
    role: String,
    ctx: Arc<ResourcesContext>,
    buf: impl Buf,
) -> Result<impl warp::Reply, Rejection> {
    // only for sellers ATM
    let role = Role::try_from(role.as_str())
        .map_err(|_| reject::custom(Error::User(UserError::UnallowedUserRole(role))))?;
    if !role.eq(&Role::Seller) {
        return Err(reject::custom(Error::User(UserError::OnlySeller)));
    }

    // check body errors
    let des = &mut serde_json::Deserializer::from_reader(buf.reader());
    let req_body: SigninRequest = serde_path_to_error::deserialize(des)
        .map_err(|e| reject::custom(Error::Request(RequestError::JSONPathError(e.to_string()))))?;

    req_body
        .validate()
        .map_err(|e| reject::custom(Error::Request(RequestError::ValidationError(e))))?;

    // format input data
    let email_lowercase = req_body.email.as_ref().map(|e| e.to_lowercase());
    let name = req_body.name.clone();
    let username = req_body.username.clone();
    let phone_number = req_body.phone_number.clone();
    let pwd = req_body.password.as_ref().map(|e| e.as_bytes());
    let pwd_hash = pwd
        .map(|pwd| hash_password(pwd))
        .transpose()
        .map_err(Error::Hash)?;

    // check for wallet_id
    let wallet_id = req_body
        .wallet_id
        .ok_or(reject::custom(Error::User(UserError::MissingWalletId)))?;

    match db_get_user_by_wallet_id(&ctx.db_client, &wallet_id).await {
        //user was found in DB
        Ok(db_user) => {
            // check for public key
            let pub_key = req_body
                .pub_key
                .ok_or(reject::custom(Error::User(UserError::MissingPubKey)))?;

            // check for signature
            let signature = req_body
                .signature
                .ok_or(reject::custom(Error::User(UserError::MissingSignature)))?;

            // check pub key on blockchain
            let account_keys = {
                let mut lock = ctx.grpc_near_client.lock().await;
                let account_keys = lock
                    .get_account_keys(&wallet_id)
                    .await
                    .map_err(|e| reject::custom(Error::Grpc(e)))?;
                drop(lock);
                account_keys
            };

            if account_keys
                .data
                .iter()
                .find(|&key| key.public_key.eq(&pub_key))
                .is_none()
            {
                return Err(reject::custom(Error::User(UserError::WrongWalletPubKey)));
            }

            // check user is a seller
            if !db_user.user_type.eq(&Role::Seller) {
                return Err(reject::custom(Error::User(UserError::OnlySeller)));
            }

            // validate signature
            let b58_encode_message = bs58::encode(&MESSAGE).into_string();
            let sig_verified = {
                let mut lock = ctx.grpc_near_client.lock().await;
                let sig_verified = lock
                    .verify_signature(&b58_encode_message, &pub_key, &signature)
                    .await
                    .map_err(|e| reject::custom(Error::Grpc(e)))?
                    .is_verified;
                drop(lock);
                sig_verified
            };

            // reject on bad signature
            if !sig_verified {
                return Err(reject::custom(Error::User(UserError::BadSignature)));
            }

            // generate and return a jwt
            let jwt_token = create_jwt(&db_user.id.to_string(), &role)
                .map_err(|e| reject::custom(Error::Auth(e)))?;

            return Ok(warp::reply::json(&SigninResponse { token: jwt_token }));
        }
        //no user with wallet_id in the db
        Err(_err) => {
            // check username is available
            if let Ok(_db_user) = db_get_user_by_username(&ctx.db_client, &req_body.username).await
            {
                return Err(reject::custom(Error::User(UserError::UnavailableUsername)));
            };

            // check email is available
            let email = email_lowercase.clone();
            if email.is_some() {
                if let Ok(_db_user) =
                    db_get_user_by_email(&ctx.db_client, &email.as_ref().expect("Email")).await
                {
                    return Err(reject::custom(Error::User(UserError::UnavailableEmail)));
                };
            }

            // check name is available
            let name = name.clone();
            if name.is_some() {
                if let Ok(_db_user) =
                    db_get_user_by_name(&ctx.db_client, &name.as_ref().expect("Name")).await
                {
                    return Err(reject::custom(Error::User(UserError::UnavailableName)));
                };
            }

            // check name is tel number (if any) is available
            let phone_number = phone_number.clone();
            if phone_number.is_some() {
                if let Ok(_db_user) = db_get_user_by_phone_number(
                    &ctx.db_client,
                    &phone_number.as_ref().expect("Phone Number"),
                )
                .await
                {
                    return Err(reject::custom(Error::User(
                        UserError::UnavailablePhoneNumber,
                    )));
                };
            }

            // NOTE: Account Id must not be implicit here, but normal!
            if !check_normal_account(&wallet_id)? {
                return Err(reject::custom(Error::User(UserError::BadNormalAccount)));
            }

            // check for present public_key and make sure it exists
            let pub_key = req_body
                .pub_key
                .ok_or(reject::custom(Error::User(UserError::MissingPubKey)))?;

            // check the submitted pub key is permissible acc. to blockchain
            let account_keys = {
                let mut lock = ctx.grpc_near_client.lock().await;
                let account_keys = lock
                    .get_account_keys(&wallet_id)
                    .await
                    .map_err(|e| reject::custom(Error::Grpc(e)))?;
                drop(lock);
                account_keys
            };

            if account_keys
                .data
                .iter()
                .find(|&key| key.public_key.eq(&pub_key))
                .is_none()
            {
                return Err(reject::custom(Error::User(UserError::WrongWalletPubKey)));
            }

            // get real account balance
            let wallet_balance = {
                let mut lock = ctx.grpc_near_client.lock().await;
                let wallet_balance = lock
                    .get_account_balance(&wallet_id)
                    .await
                    .map_err(|e| reject::custom(Error::Grpc(e)))?
                    .available;
                drop(lock);
                wallet_balance
            };

            // create a new db input user
            let new_db_user = DbUser::new(
                Uuid::new_v4(),
                name,
                username,
                phone_number,
                email_lowercase,
                pwd_hash,
                None,
                role,
                wallet_id.clone(),
                wallet_balance,
                UserStatus::PhoneVerified, // TODO: Check this for sellers ?
            );

            // insert user into db
            db_insert_user(&ctx.db_client, &new_db_user)
                .await
                .map_err(|e| reject::custom(Error::Postgres(e)))?;

            // return jwt token
            let jwt_token = create_jwt(&new_db_user.id.to_string(), &role)
                .map_err(|e| reject::custom(Error::Auth(e)))?;
            return Ok(warp::reply::json(&SigninResponse { token: jwt_token }));
        }
    }
}

// seller and admin signin with account password
pub async fn signin_with_password(
    role: String,
    ctx: Arc<ResourcesContext>,
    buf: impl Buf,
) -> Result<impl warp::Reply, Rejection> {
    // only for sellers + admins ATM
    let role = Role::try_from(role.as_str())
        .map_err(|_| reject::custom(Error::User(UserError::UnallowedUserRole(role))))?;
    let allowed_roles = vec![Role::Seller, Role::Admin];
    if !allowed_roles.contains(&role) {
        return Err(reject::custom(Error::User(UserError::UnallowedUserRole(
            role.to_string(),
        ))));
    }

    // check body errors
    let des = &mut serde_json::Deserializer::from_reader(buf.reader());
    let req_body: SigninWithPasswordRequest = serde_path_to_error::deserialize(des)
        .map_err(|e| reject::custom(Error::Request(RequestError::JSONPathError(e.to_string()))))?;

    req_body
        .validate()
        .map_err(|e| reject::custom(Error::Request(RequestError::ValidationError(e))))?;

    // get user by username
    let db_user = db_get_user_by_username(&ctx.db_client, &req_body.username)
        .await
        .map_err(|_| reject::custom(Error::User(UserError::UserNotFound)))?;

    // check user is role is allowed
    if !allowed_roles.contains(&db_user.user_type) {
        return Err(reject::custom(Error::User(UserError::UnallowedUserRole(
            db_user.user_type.to_string(),
        ))));
    }

    // get password salt from db for the user
    let db_passwd_hash: String = db_user
        .password
        .ok_or(reject::custom(Error::User(UserError::NoPassword)))?;

    let is_verified = verify_password(db_passwd_hash.as_str(), req_body.password.as_bytes())
        .map_err(|e| reject::custom(Error::Hash(e)))?;

    if !is_verified {
        return Err(reject::custom(Error::Auth(
            AuthError::WrongCredentialsError,
        )));
    }

    // generate a jwt
    let jwt_token =
        create_jwt(&db_user.id.to_string(), &role).map_err(|e| reject::custom(Error::Auth(e)))?;

    return Ok(warp::reply::json(&SigninResponse { token: jwt_token }));
}

// buyer create recovery code
pub async fn buyer_create_recovery_code(
    role: String,
    ctx: Arc<ResourcesContext>,
    buf: impl Buf,
) -> Result<impl warp::Reply, Rejection> {
    // only for buyers
    let role = Role::try_from(role.as_str())
        .map_err(|_| reject::custom(Error::User(UserError::UnallowedUserRole(role))))?;
    if !role.eq(&Role::Buyer) {
        return Err(reject::custom(Error::User(UserError::OnlyBuyer)));
    }
    // check body errors
    let des = &mut serde_json::Deserializer::from_reader(buf.reader());
    let req_body: BuyerCreateRecoveryCodeRequest = serde_path_to_error::deserialize(des)
        .map_err(|e| reject::custom(Error::Request(RequestError::JSONPathError(e.to_string()))))?;

    req_body
        .validate()
        .map_err(|e| reject::custom(Error::Request(RequestError::ValidationError(e))))?;

    // find user in the db
    let user_db = db_get_user_by_phone_number(&ctx.db_client, &req_body.phone_number)
        .await
        .map_err(|_e| reject::custom(Error::User(UserError::UserNotFound)))?;

    // generate a new recovery code
    let recovery_code: String = WasmiumRandom::secure_alphabet12()
        .into_iter()
        .take(6)
        .map(char::from)
        .collect();

    // create the sms
    let sms = SmsMessage {
        sender: None, // use the messaging service
        receiver: req_body.phone_number.clone(),
        body: Some(format!("{}{}", RECOVERY_SMS_TEXT, recovery_code.clone())),
    };

    // send recovery code via sms to buyer
    let _ = ctx
        .twilio_client
        .send_sms(&sms)
        .await
        .map_err(|e| reject::custom(Error::Twilio(e)))?;

    // create a new db buyer recovery session
    let new_db_buyer_recovery_session = DbBuyerRecoverySession::new(
        Uuid::new_v4(),
        sql_timestamp(None),
        recovery_code,
        req_body.phone_number,
        false,
        user_db.id,
    );

    // insert buyer recovery session into db
    db_insert_buyer_recovery_session(&ctx.db_client, &new_db_buyer_recovery_session)
        .await
        .map_err(|err| reject::custom(Error::Postgres(err)))?;

    // return the response
    let resp = BuyerCreateRecoveryCodeResponse::from(new_db_buyer_recovery_session);
    Ok(warp::reply::json(&resp))
}

// buyer verify recovery code
pub async fn buyer_verify_recovery_code(
    role: String,
    ctx: Arc<ResourcesContext>,
    buf: impl Buf,
) -> Result<impl warp::Reply, Rejection> {
    // only for buyers
    let role = Role::try_from(role.as_str())
        .map_err(|_| reject::custom(Error::User(UserError::UnallowedUserRole(role))))?;
    if !role.eq(&Role::Buyer) {
        return Err(reject::custom(Error::User(UserError::OnlyBuyer)));
    }

    // check body errors
    let des = &mut serde_json::Deserializer::from_reader(buf.reader());
    let req_body: BuyerVerifyRecoveryCodeRequest = serde_path_to_error::deserialize(des)
        .map_err(|e| reject::custom(Error::Request(RequestError::JSONPathError(e.to_string()))))?;

    req_body
        .validate()
        .map_err(|e| reject::custom(Error::Request(RequestError::ValidationError(e))))?;

    // parse session id
    let session_id = Uuid::parse_str(&req_body.session_id)
        .map_err(|_| Error::UnparsableUuid(req_body.session_id.clone()))?;

    // get session by id
    let mut db_buyer_recovery_session =
        db_get_buyer_recovery_session_by_id(&ctx.db_client, &session_id)
            .await
            .map_err(|_err| {
                reject::custom(Error::Session(SessionError::SessionNotFoundForUuid(
                    req_body.session_id.clone(),
                )))
            })?;

    // check the recovery code
    if !db_buyer_recovery_session
        .recovery_code
        .eq(&req_body.recovery_code)
    {
        return Err(reject::custom(Error::Session(
            SessionError::SessionRecoveryCodeMismatch(req_body.recovery_code.clone()),
        )));
    }

    // find user in the db
    let db_user = db_get_user_by_id(&ctx.db_client, &db_buyer_recovery_session.created_by_user)
        .await
        .map_err(|_e| reject::custom(Error::User(UserError::UserNotFound)))?;

    // set the session to recovered
    db_buyer_recovery_session.is_recovered = true;

    // update db
    db_update_buyer_recovery_session(&ctx.db_client, &db_buyer_recovery_session)
        .await
        .map_err(|e| reject::custom(Error::Postgres(e)))?;

    // create a new jwt
    let jwt_token =
        create_jwt(&db_user.id.to_string(), &role).map_err(|e| reject::custom(Error::Auth(e)))?;

    // return the response
    let mut resp = BuyerVerifyRecoveryCodeResponse::from(db_user);
    resp.jwt = Some(jwt_token);
    Ok(warp::reply::json(&resp))
}

// buyer register phone
pub async fn buyer_register_phone(
    role: String,
    ctx: Arc<ResourcesContext>,
    buf: impl Buf,
) -> Result<impl warp::Reply, Rejection> {
    // only for buyers
    let role = Role::try_from(role.as_str())
        .map_err(|_| reject::custom(Error::User(UserError::UnallowedUserRole(role))))?;
    if !role.eq(&Role::Buyer) {
        return Err(reject::custom(Error::User(UserError::OnlyBuyer)));
    }

    // check body errors
    let des = &mut serde_json::Deserializer::from_reader(buf.reader());
    let req_body: BuyerRegisterPhoneRequest = serde_path_to_error::deserialize(des)
        .map_err(|e| reject::custom(Error::Request(RequestError::JSONPathError(e.to_string()))))?;

    req_body
        .validate()
        .map_err(|e| reject::custom(Error::Request(RequestError::ValidationError(e))))?;

    // generate a new verification code
    let verification_code = WasmiumRandom::secure_numeric12()
        .into_iter()
        .take(6)
        .map(|item| item.to_string())
        .collect::<String>();

    // create the sms
    let sms = SmsMessage {
        sender: None, // use the messaging service
        receiver: req_body.phone_number.clone(),
        body: Some(format!(
            "{}{}",
            VERIFICATION_SMS_TEXT,
            verification_code.clone()
        )),
    };

    // send verification code via sms to buyer
    let _ = ctx
        .twilio_client
        .send_sms(&sms)
        .await
        .map_err(|e| reject::custom(Error::Twilio(e)))?;

    // create a new db buyer signup session
    let new_db_buyer_signup_session = DbBuyerSignupSession::new(
        Uuid::new_v4(),
        sql_timestamp(None),
        verification_code,
        req_body.phone_number,
        false,
    );

    // insert buyer signup session into db
    db_insert_buyer_signup_session(&ctx.db_client, &new_db_buyer_signup_session)
        .await
        .map_err(|err| reject::custom(Error::Postgres(err)))?;

    // return the response
    let resp = BuyerRegisterPhoneResponse::from(new_db_buyer_signup_session);
    Ok(warp::reply::json(&resp))
}

// buyer verify phone
pub async fn buyer_verify_phone(
    role: String,
    ctx: Arc<ResourcesContext>,
    buf: impl Buf,
) -> Result<impl warp::Reply, Rejection> {
    // only for buyers
    let role = Role::try_from(role.as_str())
        .map_err(|_| reject::custom(Error::User(UserError::UnallowedUserRole(role))))?;
    if !role.eq(&Role::Buyer) {
        return Err(reject::custom(Error::User(UserError::OnlyBuyer)));
    }

    // check body errors
    let des = &mut serde_json::Deserializer::from_reader(buf.reader());
    let req_body: BuyerVerifyPhoneRequest = serde_path_to_error::deserialize(des)
        .map_err(|e| reject::custom(Error::Request(RequestError::JSONPathError(e.to_string()))))?;

    req_body
        .validate()
        .map_err(|e| reject::custom(Error::Request(RequestError::ValidationError(e))))?;

    // parse session id
    let session_id = Uuid::parse_str(&req_body.session_id)
        .map_err(|_| Error::UnparsableUuid(req_body.session_id.clone()))?;

    // get session by id
    let mut db_buyer_signup_session =
        db_get_buyer_signup_session_by_id(&ctx.db_client, &session_id)
            .await
            .map_err(|_err| {
                reject::custom(Error::Session(SessionError::SessionNotFoundForUuid(
                    req_body.session_id.clone(),
                )))
            })?;

    // check the verification code
    if !db_buyer_signup_session
        .verification_code
        .eq(&req_body.verification_code)
    {
        return Err(reject::custom(Error::Session(
            SessionError::SessionVerificationCodeMismatch(req_body.verification_code.clone()),
        )));
    }

    // verify the session
    db_buyer_signup_session.is_verified = true;

    // update db
    db_update_buyer_signup_session(&ctx.db_client, &db_buyer_signup_session)
        .await
        .map_err(|e| reject::custom(Error::Postgres(e)))?;

    // return the response
    let resp = BuyerVerifyPhoneResponse::from(db_buyer_signup_session);
    Ok(warp::reply::json(&resp))
}

// buyer signup
pub async fn buyer_signup(
    role: String,
    ctx: Arc<ResourcesContext>,
    buf: impl Buf,
) -> Result<impl warp::Reply, Rejection> {
    // only for buyers
    let role = Role::try_from(role.as_str())
        .map_err(|_| reject::custom(Error::User(UserError::UnallowedUserRole(role))))?;
    if !role.eq(&Role::Buyer) {
        return Err(reject::custom(Error::User(UserError::OnlyBuyer)));
    }

    // check body errors
    let des = &mut serde_json::Deserializer::from_reader(buf.reader());
    let req_body: BuyerSignupRequest = serde_path_to_error::deserialize(des)
        .map_err(|e| reject::custom(Error::Request(RequestError::JSONPathError(e.to_string()))))?;

    req_body
        .validate()
        .map_err(|e| reject::custom(Error::Request(RequestError::ValidationError(e))))?;

    // check for unique username
    if db_get_users_by_username(&ctx.db_client, &req_body.username)
        .await
        .map_err(Error::Postgres)?
        .len()
        > 0
    {
        return Err(reject::custom(Error::User(UserError::UnavailableUsername)));
    }

    // parse session id
    let session_id = Uuid::parse_str(&req_body.session_id)
        .map_err(|_| Error::UnparsableUuid(req_body.session_id.clone()))?;

    // get session by id
    let db_buyer_signup_session = db_get_buyer_signup_session_by_id(&ctx.db_client, &session_id)
        .await
        .map_err(|_err| {
            reject::custom(Error::Session(SessionError::SessionNotFoundForUuid(
                req_body.session_id.clone(),
            )))
        })?;

    // check the session is verified
    if !db_buyer_signup_session.is_verified {
        return Err(reject::custom(Error::User(UserError::UnverifiedUser)));
    }

    // format input data
    let email = req_body.email.as_ref().map(|e| e.to_lowercase());
    let pwd = req_body.password.as_ref().map(|e| e.as_bytes());
    let pwd_hash = pwd
        .map(|pwd| hash_password(pwd))
        .transpose()
        .map_err(Error::Hash)?;

    // create a near implicit account
    // NOTE: the account id must have been already checked at this point
    let generated_implicit_account = {
        let mut lock = ctx.grpc_near_client.lock().await;
        let implicit_account: GenerateImplicitAccountResponse = lock
            .generate_implicit_account()
            .await
            .map_err(|e| reject::custom(Error::Grpc(e)))?;
        drop(lock);
        implicit_account
    };

    // allocate an account id
    let user_account_id = format!("{}.{}", req_body.username, NEAR_NETWORK_MODE);

    // create account and also send some funds to it (atomically)
    let create_account_status = {
        let mut lock = ctx.grpc_near_client.lock().await;
        let create_account_status: CreateAccountResponse = lock
            .create_account(
                &user_account_id,
                &generated_implicit_account.public_key,
                WALLET_CREATION_DEPOSIT_AMOUNT,
            )
            .await
            .map_err(|e| reject::custom(Error::Grpc(e)))?;
        drop(lock);
        create_account_status
    };

    if TxStatus::from_i32(create_account_status.status) == Some(TxStatus::Failed) {
        return Err(reject::custom(Error::User(UserError::WalletCreationFailed)));
    }
    log::info!(
        "Created wallet with account_id {}. Tx hash: {}",
        user_account_id,
        create_account_status.tx_hash
    );

    // send account created event over pusher TODO: spawn in a thread, error handling, retrial ???
    let _ = ctx
        .pusher_client
        .send(
            PusherChannels::Account,
            PusherEvents::AccountCreated,
            &user_account_id,
        )
        .await
        .map_err(|e| reject::custom(Error::Pusher(e)))?;

    let _ = ctx
        .pusher_client
        .send(
            PusherChannels::Account,
            PusherEvents::AccountFunded,
            &user_account_id,
        )
        .await
        .map_err(|e| reject::custom(Error::Pusher(e)))?;

    // encrypt the generated wallet secret key
    let encrypted_data = {
        let mut lock = ctx.grpc_near_client.lock().await;
        let encrypted_data: AesEncryptDataResponse = lock
            .aes_encrypt_data(&req_body.secret, &generated_implicit_account.secret_key)
            .await
            .map_err(|e| reject::custom(Error::Grpc(e)))?;
        drop(lock);
        encrypted_data
    };

    // create a new db input user (verified + store the encrypted secret key to db)
    let new_db_user = DbUser::new(
        Uuid::new_v4(),
        req_body.name,
        req_body.username.clone(),
        Some(db_buyer_signup_session.phone_number),
        email,
        pwd_hash,
        Some(encrypted_data.cypher),
        role,
        user_account_id,
        "200000000000000000000000".to_string(), // TODO: fix this with proper BigNum
        UserStatus::PhoneVerified,
    );

    // insert user into db
    db_insert_user(&ctx.db_client, &new_db_user)
        .await
        .map_err(|e| reject::custom(Error::Postgres(e)))?;

    // return the newly created user
    let jwt_token = create_jwt(&new_db_user.id.to_string(), &role)
        .map_err(|e| reject::custom(Error::Auth(e)))?;
    let mut resp = BuyerSignupResponse::from(new_db_user);
    resp.jwt = Some(jwt_token);
    resp.wallet_pub_key = Some(generated_implicit_account.public_key); // NOTE: we do not store in db the pub key!
    Ok(warp::reply::json(&resp))
}

// buyer create login code
pub async fn create_login_code(
    role: String,
    ctx: Arc<ResourcesContext>,
    buf: impl Buf,
) -> Result<impl warp::Reply, Rejection> {
    // only for buyers
    let role = Role::try_from(role.as_str())
        .map_err(|_| reject::custom(Error::User(UserError::UnallowedUserRole(role))))?;
    if !role.eq(&Role::Buyer) {
        return Err(reject::custom(Error::User(UserError::OnlyBuyer)));
    }

    // check body errors
    let des = &mut serde_json::Deserializer::from_reader(buf.reader());
    let req_body: CreateLoginCodeRequest = serde_path_to_error::deserialize(des)
        .map_err(|e| reject::custom(Error::Request(RequestError::JSONPathError(e.to_string()))))?;

    req_body
        .validate()
        .map_err(|e| reject::custom(Error::Request(RequestError::ValidationError(e))))?;

    // generate a new login token
    let login_code = WasmiumRandom::secure_numeric12()
        .into_iter()
        .take(6)
        .map(|item| item.to_string())
        .collect::<String>();

    // create a new db input session
    let expires_at = sql_timestamp(Some(5 * 60)); // set expiry in 5 minutes
    let new_db_session =
        DbSession::new(Uuid::new_v4(), expires_at, login_code.clone(), false, None);

    // insert session into db
    db_insert_session(&ctx.db_client, &new_db_session)
        .await
        .map_err(|err| reject::custom(Error::Postgres(err)))?;

    let create_login_code_response = CreateLoginCodeResponse {
        code: login_code,
        expires_at: expires_at.timestamp_millis(),
    };
    Ok(warp::reply::json(&create_login_code_response))
}

// buyer verify login code
pub async fn verify_login_code(
    role: String,
    ctx: Arc<ResourcesContext>,
    buf: impl Buf,
) -> Result<impl warp::Reply, Rejection> {
    // only for buyers
    let role = Role::try_from(role.as_str())
        .map_err(|_| reject::custom(Error::User(UserError::UnallowedUserRole(role))))?;
    if !role.eq(&Role::Buyer) {
        return Err(reject::custom(Error::User(UserError::OnlyBuyer)));
    }

    // check body errors
    let des = &mut serde_json::Deserializer::from_reader(buf.reader());
    let req_body: VerifyLoginCodeRequest = serde_path_to_error::deserialize(des)
        .map_err(|e| reject::custom(Error::Request(RequestError::JSONPathError(e.to_string()))))?;

    req_body
        .validate()
        .map_err(|e| reject::custom(Error::Request(RequestError::ValidationError(e))))?;

    // get the session by the provided login code
    let db_session = match db_get_session_by_login_code(&ctx.db_client, &req_body.code).await {
        Ok(db_session) => {
            // session found
            if db_session.is_used {
                return Err(reject::custom(Error::Session(SessionError::UsedSession(
                    req_body.code.clone(),
                ))));
            }
            if Utc::now().timestamp_millis() > db_session.expires_at.timestamp_millis() {
                return Err(reject::custom(Error::Session(
                    SessionError::ExpiredSession(req_body.code.clone()),
                )));
            }
            db_session
        }
        Err(_) => {
            // not found
            return Err(reject::custom(Error::Session(
                SessionError::NoSessionForToken(req_body.code.clone()),
            )));
        }
    };

    // validate signature
    let b58_encoded_login_code = bs58::encode(&db_session.login_code).into_string();
    let sig_verified = {
        let mut lock = ctx.grpc_near_client.lock().await;
        let sig_verified = lock
            .verify_signature(
                &b58_encoded_login_code,
                &req_body.pub_key,
                &req_body.signature,
            )
            .await
            .map_err(|e| reject::custom(Error::Grpc(e)))?
            .is_verified;
        drop(lock);
        sig_verified
    };

    // reject on bad signature
    if !sig_verified {
        return Err(reject::custom(Error::User(UserError::BadSignature)));
    }

    // get user by wallet_id
    let db_user = match db_get_user_by_wallet_id(&ctx.db_client, &req_body.wallet_id).await {
        //user was found in DB
        Ok(db_user) => {
            // check pub key in db
            let account_keys = {
                let mut lock = ctx.grpc_near_client.lock().await;
                let account_keys = lock
                    .get_account_keys(&db_user.wallet_id)
                    .await
                    .map_err(|e| reject::custom(Error::Grpc(e)))?;
                drop(lock);
                account_keys
            };

            if account_keys
                .data
                .iter()
                .find(|&key| key.public_key.eq(&req_body.pub_key))
                .is_none()
            {
                return Err(reject::custom(Error::User(UserError::WrongWalletPubKey)));
            }

            // check user is a buyer
            if !db_user.user_type.eq(&Role::Buyer) {
                return Err(reject::custom(Error::User(UserError::OnlyBuyer)));
            }

            db_user
        }
        //no user with wallet_id in the db
        Err(_err) => {
            return Err(reject::custom(Error::User(UserError::UserNotFound)));
        }
    };

    // generate a jwt
    let jwt_token =
        create_jwt(&db_user.id.to_string(), &role).map_err(|e| reject::custom(Error::Auth(e)))?;

    // update db session record
    let _is_success = db_update_session_info(&ctx.db_client, &db_session.id, &db_user.id, true)
        .await
        .map_err(|e| reject::custom(Error::Postgres(e)))?;

    // send jwt over pusher TODO: spawn in a thread, error handling, retrial ???
    let events = ctx
        .pusher_client
        .send(
            PusherChannels::Custom(db_session.login_code),
            PusherEvents::LoggedIn,
            &jwt_token,
        )
        .await
        .map_err(|e| reject::custom(Error::Pusher(e)))?;

    log::info!("Successfully sent login event: {:?}", events);

    let verify_login_code_response = VerifyLoginCodeResponse {};
    Ok(warp::reply::json(&verify_login_code_response))
}

// buyer gets an event verification code
pub async fn event_ticket_get_verification_code(
    role: String,
    ctx: Arc<ResourcesContext>,
    buf: impl Buf,
    user_id: uuid::Uuid, // authenticated user id calling the endpoint
) -> Result<impl warp::Reply, Rejection> {
    // only for buyers ATM
    let role = Role::try_from(role.as_str())
        .map_err(|_| reject::custom(Error::User(UserError::UnallowedUserRole(role))))?;
    if !role.eq(&Role::Buyer) {
        return Err(reject::custom(Error::User(UserError::OnlyBuyer)));
    }

    // check body errors
    let des = &mut serde_json::Deserializer::from_reader(buf.reader());
    let req_body: EventTicketGetVerificationCodeRequest = serde_path_to_error::deserialize(des)
        .map_err(|e| reject::custom(Error::Request(RequestError::JSONPathError(e.to_string()))))?;

    req_body
        .validate()
        .map_err(|e| reject::custom(Error::Request(RequestError::ValidationError(e))))?;

    // generate verification code
    let verification_code = WasmiumRandom::secure_numeric12()
        .into_iter()
        .take(6)
        .map(|item| item.to_string())
        .collect::<String>();

    // get the event by id
    let event_id = Uuid::parse_str(&req_body.event_id)
        .map_err(|_| Error::UnparsableUuid(req_body.event_id.clone()))?;
    let db_event = db_get_event_by_id(&ctx.db_client, &event_id)
        .await
        .map_err(|_| {
            reject::custom(Error::Event(EventError::NoExistEventUuid(
                event_id.to_string(),
            )))
        })?;

    // loop over reservations and add them to db
    for reservation in req_body.reservations.into_iter() {
        // get ticket id
        let ticket_id = Uuid::parse_str(&reservation.ticket_id)
            .map_err(|_| Error::UnparsableUuid(reservation.ticket_id.clone()))?;

        // find the ticket by uuid in the db
        let db_ticket = db_get_ticket_by_id(&ctx.db_client, &ticket_id)
            .await
            .map_err(|_| {
                reject::custom(Error::Ticket(TicketError::NoExistTicketUuid(
                    ticket_id.to_string(),
                )))
            })?;

        // check event id for ticket corresponds to event id
        if db_ticket.event_id.ne(&event_id) {
            return Err(reject::custom(Error::Ticket(
                TicketError::TicketEventMismatch(event_id.to_string()),
            )));
        }

        // check there are no other reservations for this (event_id, ticket_id, user_id, code)
        let ticket_reservations = db_get_ticket_reservations_by_user_id(&ctx.db_client, &user_id)
            .await
            .map_err(|e| reject::custom(Error::Postgres(e)))?;

        if ticket_reservations.iter().any(|reservation| {
            reservation.ticket_id.eq(&ticket_id)
                && reservation.event_id.eq(&event_id)
                && reservation.user_id.eq(&user_id)
                && reservation.verification_code.eq(&verification_code)
        }) {
            return Err(reject::custom(Error::Ticket(
                TicketError::AlreadyReservedForUser(user_id.to_string()),
            )));
        }

        // create a new db ticket reservation
        let new_db_ticket_reservation = DbTicketReservation::new(
            Uuid::new_v4(),
            sql_timestamp(None),
            &verification_code,
            db_event.id,
            ticket_id,
            user_id,
        );

        // insert ticket reservation into db
        db_insert_ticket_reservation(&ctx.db_client, &new_db_ticket_reservation)
            .await
            .map_err(|e| reject::custom(Error::Postgres(e)))?;
    }

    return Ok(warp::reply::json(&EventGetVerificationCodeResponse {
        verification_code,
    }));
}

// buyer gets an event verification code
pub async fn get_event_from_verification_code(
    role: String,
    ctx: Arc<ResourcesContext>,
    buf: impl Buf,
    user_id: uuid::Uuid, // authenticated user id calling the endpoint
) -> Result<impl warp::Reply, Rejection> {
    // only for buyers ATM
    let role = Role::try_from(role.as_str())
        .map_err(|_| reject::custom(Error::User(UserError::UnallowedUserRole(role))))?;
    if !role.eq(&Role::Buyer) {
        return Err(reject::custom(Error::User(UserError::OnlyBuyer)));
    }

    // check body errors
    let des = &mut serde_json::Deserializer::from_reader(buf.reader());
    let req_body: GetEventFromVerificationCodeRequest = serde_path_to_error::deserialize(des)
        .map_err(|e| reject::custom(Error::Request(RequestError::JSONPathError(e.to_string()))))?;

    req_body
        .validate()
        .map_err(|e| reject::custom(Error::Request(RequestError::ValidationError(e))))?;

    // get ticket reservations by code
    let db_ticket_reservations =
        db_get_ticket_reservations_by_code(&ctx.db_client, &req_body.verification_code)
            .await
            .map_err(|e| reject::custom(Error::Postgres(e)))?;

    // if no ticket reservations, return error
    if db_ticket_reservations.is_empty() {
        return Err(reject::custom(Error::Ticket(
            TicketError::NoTicketReservationsForCode(req_body.verification_code.to_string()),
        )));
    }

    // check if all reservations for the code belong to the same calling user
    if db_ticket_reservations
        .iter()
        .any(|reservation| reservation.user_id.ne(&user_id))
    {
        return Err(reject::custom(Error::Ticket(
            TicketError::WrongUserReserved(user_id.to_string()),
        )));
    }

    // check if all reservations for the code belong to the same event id
    let event_id = db_ticket_reservations
        .get(0)
        .expect("A valid ticket reservation")
        .event_id;

    if db_ticket_reservations
        .iter()
        .any(|reservation| reservation.event_id.ne(&event_id))
    {
        return Err(reject::custom(Error::Ticket(
            TicketError::TicketEventMismatch(event_id.to_string()),
        )));
    }

    // get the event from the reservation and fetch data
    let db_event = db_get_event_by_id(&ctx.db_client, &event_id)
        .await
        .map_err(|_| {
            reject::custom(Error::Event(EventError::NoExistEventUuid(
                event_id.to_string(),
            )))
        })?;

    // get event tickets
    let tickets = db_get_tickets_by_event_id(&ctx.db_client, &Some(db_event.id))
        .await
        .map_err(|e| reject::custom(Error::Postgres(e)))?;

    return Ok(warp::reply::json(
        &GetEventFromVerificationCodeResponse::new(db_event, tickets),
    ));
}
