# start the service locally if you want to test
RUST_BACKTRACE=full ENV=dev RUST_LOG=info,gql,gqli cargo run --bin gql-api -- --config ./config.toml

# start the test client locally if you want to test
RUST_BACKTRACE=full PROTO_DIR="../../protos/nearapiservice.proto" AWS_CONFIG_FILE="~/.aws/config" AWS_SHARED_CREDENTIALS_FILE="~/.aws/credentials" AWS_PROFILE=default  ENV=dev RUST_LOG=info,gql,gqli cargo run --bin gql-api -- --config ./config.toml

# deploy gql-api to aws registry
DOCKER_BUILDKIT=1 docker build --file Dockerfile --tag gql-api --target release --build-arg BUILD_ENV=release --build-arg RUST_VERSION=stable --build-arg RUSTC_WRAPPER="sccache" .
aws ecr get-login-password --region eu-west-2 | docker login --username AWS --password-stdin 382480309488.dkr.ecr.eu-west-2.amazonaws.com
docker tag gql-api:latest 382480309488.dkr.ecr.eu-west-2.amazonaws.com/gql-api:latest
docker push 382480309488.dkr.ecr.eu-west-2.amazonaws.com/gql-api:latest