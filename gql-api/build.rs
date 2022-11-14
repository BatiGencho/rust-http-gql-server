use std::env;

const DEFAULT_IMAGE_PROTO_DIR: &str = "../../protos/nearapiservice.proto";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = env::var("PROTO_DIR").unwrap_or_else(|_| DEFAULT_IMAGE_PROTO_DIR.to_string());
    tonic_build::compile_protos(proto_dir)
        .unwrap_or_else(|e| panic!("Failed to compile near api proto {:?}", e));
    Ok(())
}
