pub mod types;

/// Module for accessing all blockchain related types. It is based on the encoded metadata provided at `runtime_metadata_path`
#[subxt::subxt(runtime_metadata_path = "chain-metadata.scale")]
pub mod titanh {}
