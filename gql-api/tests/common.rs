use chrono::{Local, Utc};
use gql_api::{
    auth::{Role, UserStatus},
    config::{db_client_from_config, PostgresConfig},
    db::models::{AssetFile, DbEvent, DbUser},
    gql::models::EventStatus,
};
use rand::Rng;
use tokio_postgres::Client;

pub struct TestContext {
    pub client: Client,
    pub event: DbEvent,
}

pub async fn setup() -> TestContext {
    let (db_client, connection) = db_client_from_config(&PostgresConfig {
        db_host: "127.0.0.1".to_string(),
        db_port: 5432,
        db_name: "usersdb".to_string(),
        db_user: "postgres".to_string(),
        db_pwd: "postgres".to_string(),
    })
    .await
    .expect("unable to establish a db connection");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let event = create_event(&db_client).await;

    TestContext {
        client: db_client,
        event,
    }
}

pub async fn create_event(db_client: &Client) -> DbEvent {
    let event_name = gen_string(20);
    let now = Local::now();

    let user_id = uuid::Uuid::new_v4();

    gql_api::db::sql::db_insert_user(
        &db_client,
        &DbUser {
            id: user_id,
            name: None,
            username: gen_string(20),
            phone_number: None,
            email: None,
            password: None,
            encrypted_secret_key: None,
            created_at: Utc::now().naive_utc(),
            wallet_id: gen_string(20),
            wallet_balance: "0".to_string(),
            user_type: Role::Seller,
            user_status: UserStatus::Unverified,
        },
    )
    .await
    .expect("unable to create user");

    gql_api::db::sql::db_insert_event(
        &db_client,
        &DbEvent {
            event_name: event_name.clone(),
            is_virtual: Some(true),
            is_featured: Some(false),
            venue_name: Some(gen_string(20)),
            event_slug: gen_string(20),
            id: uuid::Uuid::new_v4(),
            created_at: now.naive_utc(),
            start_date: Some(now.naive_utc()),
            end_date: None,
            entry_time: None,
            description: Some(gen_string(20)),
            venue_location: Some(gen_string(20)),
            cover_photo_url: None,
            thumbnail_url: None,
            event_status: EventStatus::Draft,
            created_by_user: user_id,
        },
    )
    .await
    .expect("unable to create event");

    gql_api::db::sql::db_get_event_by_name(db_client, &event_name)
        .await
        .expect("unable to fetch event")
}

pub fn gen_asset_file(bucket: impl Into<String>, event_id: uuid::Uuid) -> AssetFile {
    AssetFile {
        id: uuid::Uuid::new_v4(),
        s3_bucket: bucket.into(),
        s3_absolute_key: format!("{}/{}.{}", gen_string(10), gen_string(10), gen_string(3)),
        ipfs_hash: None,
        event_id,
    }
}

pub fn gen_string(length: usize) -> String {
    use rand::distributions::Alphanumeric;
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
