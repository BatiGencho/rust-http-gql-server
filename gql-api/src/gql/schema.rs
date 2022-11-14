use crate::{
    gql::{
        mutations::{PrivateMutationRoot, PublicMutationRoot},
        quiries::{PrivateQueryRoot, PublicQueryRoot},
        subscriptions::{PrivateSubscriptionRoot, PublicSubscriptionRoot},
    },
    grpc::GrpcNearClient,
};
use juniper::RootNode;
use pusher_client::client::PusherClient;
use s3_uploader::{s3::S3Client, AwsContext};
use tokio::sync::Mutex;
use tokio_postgres::Client;
use twilio_client::client::TwilioClient;
use uuid::Uuid;

pub type PublicSchema =
    RootNode<'static, PublicQueryRoot, PublicMutationRoot, PublicSubscriptionRoot>;
pub type PrivateSchema =
    RootNode<'static, PrivateQueryRoot, PrivateMutationRoot, PrivateSubscriptionRoot>;

pub struct Context {
    pub db_client: Client,
    pub grpc_near_client: Mutex<GrpcNearClient>,
    pub user_id: Mutex<Option<Uuid>>,
    pub pusher_client: PusherClient,
    pub twilio_client: TwilioClient,
    pub aws_s3_client: S3Client,
    pub aws_context: AwsContext,
}

impl juniper::Context for Context {}
