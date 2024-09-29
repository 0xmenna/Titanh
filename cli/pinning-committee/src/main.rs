use api::TitanhApiBuilder;
use clap::{Parser, Subcommand};
use std::fs::{self};
use std::path::PathBuf;

pub const CHAIN_ENDPOINT: &str = "ws://127.0.0.1:9944";

#[derive(Parser)]
#[command(name = "pinning-committee-cli")]
#[command(about = "CLI tool to manage on chain operations for the pinning committee")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Set the pinning committee configuration
    CommitteeConfig {
        /// The seed phrase of the sudo account
        #[arg(short, long)]
        seed_phrase: String,
        /// The content replication factor
        #[arg(short, long)]
        rep_factor: u32,
        /// The number of IPFS replicas for each pinning node
        #[arg(short, long)]
        ipfs_replicas: u32,
        /// The number of pinning nodes per validator
        #[arg(short, long)]
        pinning_nodes: u32,
    },
    /// Register a new pinning node
    RegisterPinningNode {
        /// The seed phrase of the validator account
        #[arg(short, long)]
        seed_phrase: String,
        /// The path containing the private keys of the IPFS nodes in PEM format
        #[arg(short, long)]
        privkeys: Option<String>,
    },
}

fn get_seeds_from_pem_files(
    directory: Option<PathBuf>,
) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
    // Determine the directory to search
    let dir_path = directory.unwrap_or_else(|| std::env::current_dir().unwrap());

    let mut seeds = Vec::new();

    // Iterate over entries in the directory
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        // Check if it's a `.pem` file
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("pem") {
            // Read the PEM file contents
            let pem_contents = fs::read_to_string(&path)?;
            // Parse the PEM content
            let pem = pem::parse(pem_contents)?;
            if pem.contents().len() >= 16 {
                // Extract the seed starting from the 17th byte
                let seed = pem.contents()[16..].to_vec();
                seeds.push(seed);
            } else {
                return Err(From::from(format!(
                    "Content of PEM file {} is too short",
                    path.display()
                )));
            }
        }
    }

    Ok(seeds)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::CommitteeConfig {
            seed_phrase,
            rep_factor,
            ipfs_replicas,
            pinning_nodes,
        } => {
            let api = TitanhApiBuilder::rpc(CHAIN_ENDPOINT)
                .seed(&seed_phrase)
                .build()
                .await;

            let tx_hash = api
                .pinning_committee()
                .set_committe_config(rep_factor, ipfs_replicas, pinning_nodes)
                .await?;
            println!(
                "Committee configuration transaction was successful. Transaction hash: {:?}",
                tx_hash
            );
        }
        Commands::RegisterPinningNode {
            seed_phrase,
            privkeys,
        } => {
            let privkeys_path = privkeys.map(PathBuf::from);
            let ipfs_seeds = get_seeds_from_pem_files(privkeys_path)?;

            let api = TitanhApiBuilder::rpc(CHAIN_ENDPOINT)
                .seed(&seed_phrase)
                .build()
                .await;

            let tx_hash = api
                .pinning_committee()
                .ipfs_seeds(ipfs_seeds)?
                .register_ipfs_peers()
                .await?;
            println!(
                "Registering pinning node transaction was successful. Transaction hash: {:?}",
                tx_hash
            );
        }
    }

    Ok(())
}
