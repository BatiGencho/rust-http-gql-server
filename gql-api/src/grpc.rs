use self::near_api::{
    AesDecryptDataRequest, AesDecryptDataResponse, AesEncryptDataRequest, AesEncryptDataResponse,
    CheckAvailableAccountIdRequest, CheckAvailableAccountIdResponse, CreateAccountRequest,
    CreateAccountResponse, GenerateImplicitAccountRequest, GenerateImplicitAccountResponse,
    GetAccountKeysRequest, GetAccountKeysResponse, MintNftsRequest, MintNftsResponse,
    VerifySignatureRequest, VerifySignatureResponse,
};
use crate::config::GrpcConfig;
use crate::error::GrpcError;
use near_api::near_api_engine_service_client::NearApiEngineServiceClient;
use near_api::{
    FundAccountRequest, FundAccountResponse, GetAccountBalanceRequest, GetAccountBalanceResponse,
};
pub mod near_api {
    tonic::include_proto!("com.project.near"); // this is the proto package name
}

pub struct GrpcNearClient {
    near_api_client: NearApiEngineServiceClient<tonic::transport::channel::Channel>,
}

pub async fn new(config: &GrpcConfig) -> Result<GrpcNearClient, GrpcError> {
    let grpc_server_addr = match config.tls {
        Some(_) => {
            format!("https://{}:{}", config.bind_host, config.bind_port)
        }
        None => {
            format!("http://{}:{}", config.bind_host, config.bind_port)
        }
    };
    /*
    let channel = tonic::transport::Channel::from_shared(grpc_addr)
        .map_err(GrpcError::Uri)?
        .connect()
        .await
        .map_err(GrpcError::Transport)?;
    let near_api_client = NearApiEngineServiceClient::new(channel);
    */
    let near_api_client = NearApiEngineServiceClient::connect(grpc_server_addr)
        .await
        .map_err(GrpcError::Transport)?;
    Ok(GrpcNearClient { near_api_client })
}

impl GrpcNearClient {
    pub async fn get_account_balance(
        &mut self,
        account_id: &str,
    ) -> Result<GetAccountBalanceResponse, GrpcError> {
        let request = tonic::Request::new(GetAccountBalanceRequest {
            account_id: account_id.into(),
        });
        match self.near_api_client.get_account_balance(request).await {
            Ok(response) => {
                let response = response.into_inner();
                return Ok(response);
            }
            Err(status) => {
                return Err(GrpcError::Call(status));
            }
        }
    }

    pub async fn fund_account(
        &mut self,
        account_id: &str,
        fund_amount: &str,
    ) -> Result<FundAccountResponse, GrpcError> {
        let request = tonic::Request::new(FundAccountRequest {
            account_id: account_id.into(),
            amount: fund_amount.into(),
        });
        match self.near_api_client.fund_account(request).await {
            Ok(response) => {
                let response = response.into_inner();
                return Ok(response);
            }
            Err(status) => {
                return Err(GrpcError::Call(status));
            }
        }
    }

    pub async fn create_account(
        &mut self,
        account_id: &str,
        public_key: &str,
        deposit_amount: &str,
    ) -> Result<CreateAccountResponse, GrpcError> {
        let request = tonic::Request::new(CreateAccountRequest {
            account_id: account_id.into(),
            public_key: public_key.into(),
            deposit_amount: deposit_amount.into(),
        });
        match self.near_api_client.create_account(request).await {
            Ok(response) => {
                let response = response.into_inner();
                return Ok(response);
            }
            Err(status) => {
                return Err(GrpcError::Call(status));
            }
        }
    }

    pub async fn mint_nfts(
        &mut self,
        seller_wallet_id: String,
        title: String,
        ticket_slug: String,
        description: String,
        media: String,
        media_hash: String,
        number_of_tickets: i32,
        extra: String,
        amount_to_send: String,
    ) -> Result<MintNftsResponse, GrpcError> {
        let request = tonic::Request::new(MintNftsRequest {
            seller_wallet_id,
            title,
            ticket_slug,
            description,
            media,
            media_hash,
            number_of_tickets,
            extra,
            amount_to_send,
        });
        match self.near_api_client.mint_nfts(request).await {
            Ok(response) => {
                let response = response.into_inner();
                return Ok(response);
            }
            Err(status) => {
                return Err(GrpcError::Call(status));
            }
        }
    }

    pub async fn check_available_account_id(
        &mut self,
        account_id: &str,
    ) -> Result<CheckAvailableAccountIdResponse, GrpcError> {
        let request = tonic::Request::new(CheckAvailableAccountIdRequest {
            account_id: account_id.into(),
        });
        match self
            .near_api_client
            .check_available_account_id(request)
            .await
        {
            Ok(response) => {
                let response = response.into_inner();
                return Ok(response);
            }
            Err(status) => {
                return Err(GrpcError::Call(status));
            }
        }
    }

    pub async fn generate_implicit_account(
        &mut self,
    ) -> Result<GenerateImplicitAccountResponse, GrpcError> {
        let request = tonic::Request::new(GenerateImplicitAccountRequest {});
        match self
            .near_api_client
            .generate_implicit_account(request)
            .await
        {
            Ok(response) => {
                let response = response.into_inner();
                return Ok(response);
            }
            Err(status) => {
                return Err(GrpcError::Call(status));
            }
        }
    }

    pub async fn verify_signature(
        &mut self,
        message: &str,
        pub_key: &str,
        signature: &str,
    ) -> Result<VerifySignatureResponse, GrpcError> {
        let request = tonic::Request::new(VerifySignatureRequest {
            message: message.into(),
            pub_key: pub_key.into(),
            signature: signature.into(),
        });
        match self.near_api_client.verify_signature(request).await {
            Ok(response) => {
                let response = response.into_inner();
                return Ok(response);
            }
            Err(status) => {
                return Err(GrpcError::Call(status));
            }
        }
    }

    pub async fn get_account_keys(
        &mut self,
        account_id: &str,
    ) -> Result<GetAccountKeysResponse, GrpcError> {
        let request = tonic::Request::new(GetAccountKeysRequest {
            account_id: account_id.into(),
        });
        match self.near_api_client.get_account_keys(request).await {
            Ok(response) => {
                let response = response.into_inner();
                return Ok(response);
            }
            Err(status) => {
                return Err(GrpcError::Call(status));
            }
        }
    }

    pub async fn aes_encrypt_data(
        &mut self,
        secret: &str,
        data: &str,
    ) -> Result<AesEncryptDataResponse, GrpcError> {
        let request = tonic::Request::new(AesEncryptDataRequest {
            secret: secret.into(),
            data: data.into(),
        });
        match self.near_api_client.aes_encrypt_data(request).await {
            Ok(response) => {
                let response = response.into_inner();
                return Ok(response);
            }
            Err(status) => {
                return Err(GrpcError::Call(status));
            }
        }
    }

    pub async fn aes_decrypt_data(
        &mut self,
        cypher: &str,
        secret: &str,
    ) -> Result<AesDecryptDataResponse, GrpcError> {
        let request = tonic::Request::new(AesDecryptDataRequest {
            cypher: cypher.into(),
            secret: secret.into(),
        });
        match self.near_api_client.aes_decrypt_data(request).await {
            Ok(response) => {
                let response = response.into_inner();
                return Ok(response);
            }
            Err(status) => {
                return Err(GrpcError::Call(status));
            }
        }
    }
}
