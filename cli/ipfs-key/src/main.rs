use base64::engine::general_purpose;
use base64::Engine;
use clap::{Parser, Subcommand};
use libp2p::identity::{ed25519::Keypair, Keypair as IdKeypair};

#[derive(Parser)]
#[command(name = "ipfs-key")]
#[command(
    about = "CLI tool to extract public key or sign messages using Ed25519 private key in PEM format"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate ipfs peer infos: seed, privkey_protobuf, pubkey, peer_id
    Generate {},
}

fn generate_peer_info() -> Result<PeerInfo, String> {
    // Generate the Keypair
    let keypair = Keypair::generate();
    let seed = keypair.secret();
    let seed = seed.as_ref();
    let pubkey = keypair.public().to_bytes();

    let keypair = IdKeypair::from(keypair);

    // Serialize the Keypair to protobuf encoding
    let protobuf_bytes = keypair
        .to_protobuf_encoding()
        .map_err(|e| format!("Failed to serialize Keypair: {}", e))?;

    // Base64-encode the serialized private key
    let encoded_privkey = general_purpose::STANDARD.encode(&protobuf_bytes);

    let peer_id = keypair.public().to_peer_id();

    let peer_info = PeerInfo {
        seed: seed.to_vec(),
        privkey_protobuf: encoded_privkey,
        pubkey: pubkey.to_vec(),
        peer_id: peer_id.to_string(),
    };

    Ok(peer_info)
}

struct PeerInfo {
    seed: Vec<u8>,
    privkey_protobuf: String,
    pubkey: Vec<u8>,
    peer_id: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate {} => {
            let peer_info = generate_peer_info()?;

            println!("Seed: 0x{}", hex::encode(peer_info.seed));
            println!("Privkey_protobuf: {}", peer_info.privkey_protobuf);
            println!("Pubkey: 0x{}", hex::encode(peer_info.pubkey));
            println!("Peer_id: {}", peer_info.peer_id);
        }
    }

    Ok(())
}
