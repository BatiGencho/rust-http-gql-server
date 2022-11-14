use anyhow::{Context, Result};
use argh::{self, FromArgs};
use gql_api::config::{db_client_from_config, Config, ServerEnv};
use gql_api::error::{handle_rejection, Error};
use gql_api::filters::with_cors;
use gql_api::gql::{
    mutations::{PrivateMutationRoot, PublicMutationRoot},
    quiries::{PrivateQueryRoot, PublicQueryRoot},
    routes::{graphql_private_route, graphql_public_route, public_graphiql_route},
    schema::{Context as ResourcesContext, PrivateSchema, PublicSchema},
    subscriptions::{PrivateSubscriptionRoot, PublicSubscriptionRoot},
};
use gql_api::http::routes::{
    buyer_create_recovery_code_route, buyer_register_phone_route, buyer_signup_route,
    buyer_verify_phone_route, buyer_verify_recovery_code_route, check_username_route,
    create_login_code_route, event_ticket_get_verification_code_route,
    get_event_from_verification_code_route, healthcheck_route, homepage_route, signin_route,
    signin_with_password_route, verify_login_code_route,
};
use pusher_client::client::PusherClient;
use s3_uploader::DEFAULT_REGION;
use s3_uploader::{s3::S3Client, AwsContext};
use std::{env, net::SocketAddr, sync::Arc};
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::{broadcast, Mutex};
use twilio_client::client::TwilioClient;
use warp::Filter;

#[tokio::main]
async fn main() -> Result<()> {
    // init config
    dotenv::dotenv().ok();
    let args: Args = argh::from_env();

    let config = Config::new(args.config)
        .await
        .context("Failed to load config")?;

    // init logging
    pretty_env_logger::init();
    env::set_var("RUST_LOG", "info,gql,gqli,http");
    let env = env::var("ENV").context("Failed to read the ENV variable")?;
    let server_env = ServerEnv::from_str(&env);
    let graphql_logger = warp::log("gql");
    let graphiql_logger = warp::log("gqli");
    let http_logger = warp::log("http");

    // stop signals
    let (stop_tx, mut stop_rx) = broadcast::channel(1);
    tokio::spawn(stop_signal(stop_tx.clone()));

    gql_api::migrations::run(&config.postgres);

    let (db_client, connection) = db_client_from_config(&config.postgres)
        .await
        .expect("unable to establish a db connection");

    let db_stop_tx = stop_tx.clone();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            log::error!("DB Connection Error: {}", e);
            db_stop_tx
                .send(())
                .expect("error sending a db stop message");
        }
    });

    // server url
    let server_addr = format!("{}:{}", config.api.bind_host, config.api.bind_port)
        .parse::<SocketAddr>()
        .map_err(Error::ParseAddr)?;

    // Create gql schema
    let public_gql_schema = Arc::new(PublicSchema::new(
        PublicQueryRoot,
        PublicMutationRoot,
        PublicSubscriptionRoot,
    ));
    let private_gql_schema = Arc::new(PrivateSchema::new(
        PrivateQueryRoot,
        PrivateMutationRoot,
        PrivateSubscriptionRoot,
    ));

    // create grpc client for near
    let grpc_near_client = gql_api::grpc::new(&config.near_api)
        .await
        .map_err(Error::Grpc)?;

    // create pusher client (NOTE: this trick is required as the latest builder in the lib does not support clusters!)
    let pusher_client = PusherClient::new(&config.pusher).map_err(Error::Pusher)?;

    // create aws client
    let aws_client_ctx = AwsContext::build(
        config.s3.region.or(Some(DEFAULT_REGION.to_string())),
        config.s3.bucket,
        config.s3.prefix,
    )
    .await;
    let aws_s3_client = S3Client::new_from_context(&aws_client_ctx);

    // create twilio config
    let twilio_client = TwilioClient::new(config.twilio.api.clone(), config.twilio.sms.clone())
        .map_err(Error::Twilio)?;

    // Create context
    let resources_ctx = Arc::new(ResourcesContext {
        db_client,
        grpc_near_client: Mutex::new(grpc_near_client),
        user_id: Mutex::new(None),
        pusher_client,
        twilio_client,
        aws_s3_client,
        aws_context: aws_client_ctx,
    });

    // unprotected routes
    let check_username_route = check_username_route(resources_ctx.clone(), http_logger);
    let healthcheck_route = healthcheck_route(resources_ctx.clone(), http_logger);
    let _homepage_route = homepage_route(http_logger);

    // buyer http routes
    let buyer_signup_route = buyer_signup_route(resources_ctx.clone(), http_logger);
    let buyer_register_phone_route = buyer_register_phone_route(resources_ctx.clone(), http_logger);
    let buyer_verify_phone_route = buyer_verify_phone_route(resources_ctx.clone(), http_logger);
    let buyer_create_recovery_code_route =
        buyer_create_recovery_code_route(resources_ctx.clone(), http_logger);
    let buyer_verify_recovery_code_route =
        buyer_verify_recovery_code_route(resources_ctx.clone(), http_logger);

    // seller http routes
    let signin_route = signin_route(resources_ctx.clone(), http_logger);
    let signin_with_password_route = signin_with_password_route(resources_ctx.clone(), http_logger);
    let create_login_code_route = create_login_code_route(resources_ctx.clone(), http_logger);
    let verify_login_code_route = verify_login_code_route(resources_ctx.clone(), http_logger);
    let event_ticket_get_verification_code =
        event_ticket_get_verification_code_route(resources_ctx.clone(), http_logger);
    let get_event_from_verification_code =
        get_event_from_verification_code_route(resources_ctx.clone(), http_logger);

    // create gql routes (protected and unprotected)
    let graphql_private_route = graphql_private_route(
        resources_ctx.clone(),
        private_gql_schema.clone(),
        graphql_logger,
    );
    let graphql_public_route = graphql_public_route(
        resources_ctx.clone(),
        public_gql_schema.clone(),
        graphql_logger,
    );

    // public graphiql route (only for DEV purposes, in fact not required as we have a POSTMAN collection!)
    let _public_graphiql_route = public_graphiql_route(server_addr.clone(), graphiql_logger);

    // bundle routes
    let routes = check_username_route
        .or(healthcheck_route)
        .or(buyer_signup_route)
        .or(buyer_register_phone_route)
        .or(buyer_verify_phone_route)
        .or(signin_route)
        .or(signin_with_password_route)
        .or(buyer_create_recovery_code_route)
        .or(buyer_verify_recovery_code_route)
        .or(create_login_code_route)
        .or(verify_login_code_route)
        .or(event_ticket_get_verification_code)
        .or(get_event_from_verification_code)
        .or(graphql_private_route)
        .or(graphql_public_route)
        .with(with_cors())
        .recover(handle_rejection);

    // run the server
    match server_env {
        ServerEnv::Dev => {
            // dev mode: no certs needed
            let (_addr, server) =
                warp::serve(routes).bind_with_graceful_shutdown(server_addr, async move {
                    log::info!("waiting for a signal...");
                    _ = stop_rx.recv().await;

                    log::info!("cleaning up resources..."); //TODO ???
                    log::info!("done cleaning resources!");
                    log::info!("Exiting...!")
                });
            log::info!("Graphql server listening on {}", server_addr.to_string());
            match tokio::join!(tokio::task::spawn(server)).0 {
                Ok(()) => log::info!(""),
                Err(e) => log::info!("Main Thread join error {}", e),
            };
        }
        ServerEnv::Release => {
            let cert = config.api.tls.ok_or(Error::MissingCertificate)?;
            log::info!("Graphql server mounting certificates {:?}", cert);

            //release mode: add certs
            let (_addr, server) = warp::serve(routes)
                .tls()
                .cert_path(cert.certificate)
                .key_path(cert.private_key)
                .bind_with_graceful_shutdown(server_addr, async move {
                    log::info!("waiting for a signal...");
                    _ = stop_rx.recv().await;

                    log::info!("cleaning up resources..."); //TODO ???
                    log::info!("done cleaning resources!");
                    log::info!("Exiting...!")
                });
            log::info!("Graphql server listening on {}", server_addr.to_string());
            match tokio::join!(tokio::task::spawn(server)).0 {
                Ok(()) => log::info!(""),
                Err(e) => log::info!("Main Thread join error {}", e),
            };
        }
    }

    Ok(())
}

async fn stop_signal(stop_tx: broadcast::Sender<()>) {
    let _ = signal(SignalKind::terminate())
        .expect("shutdown_listener")
        .recv()
        .await;

    log::info!("Received shutdown signal...");
    let _ = stop_tx.send(());
}

/// Events Service
#[derive(FromArgs)]
struct Args {
    /// path to the config file
    #[argh(option, short = 'c')]
    config: String,
}
