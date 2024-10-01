use api::TitanhApiBuilder;
use clap::{Parser, Subcommand};
use std::fs;
use std::io::BufRead;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "pinning-committee-cli")]
#[command(about = "CLI tool to manage on-chain operations for the pinning committee")]
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
        /// The chain rpc endpoint
        #[arg(short, long)]
        rpc: String,
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
        /// The chain rpc endpoint
        #[arg(short, long)]
        rpc: String,
        /// The path to the file containing hex-encoded IPFS seeds, one per line
        #[arg(short, long)]
        seeds_file: String,
    },
}

/// Reads a single file containing hex-encoded seeds, one per line.
/// Returns a vector of seed byte arrays.
fn get_seeds_from_hex_file(path: PathBuf) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
    let file = fs::File::open(&path).map_err(|e| {
        format!(
            "Failed to open seeds file {}: {}",
            path.display(),
            e.to_string()
        )
    })?;
    let reader = std::io::BufReader::new(file);

    let mut seeds = Vec::new();

    for (idx, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| {
            format!(
                "Failed to read line {} in seeds file {}: {}",
                idx + 1,
                path.display(),
                e.to_string()
            )
        })?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue; // Skip empty lines
        }
        let seed = hex::decode(trimmed).map_err(|e| {
            format!(
                "Failed to decode hex on line {} in seeds file {}: {}",
                idx + 1,
                path.display(),
                e.to_string()
            )
        })?;
        seeds.push(seed);
    }

    Ok(seeds)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::CommitteeConfig {
            seed_phrase,
            rpc,
            rep_factor,
            ipfs_replicas,
            pinning_nodes,
        } => {
            let api = TitanhApiBuilder::rpc(&rpc).seed(&seed_phrase).build().await;

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
            rpc,
            seeds_file,
        } => {
            let seeds_path = PathBuf::from(seeds_file);
            let ipfs_seeds = get_seeds_from_hex_file(seeds_path)?;

            let api = TitanhApiBuilder::rpc(&rpc).seed(&seed_phrase).build().await;

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
