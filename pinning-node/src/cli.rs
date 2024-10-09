use clap::{Parser, Subcommand};

use crate::utils::config::{Config, PeersConfig};

#[derive(Parser)]
#[command(name = "pinning-node")]
#[command(about = "CLI to start a pinning node")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    pub fn parse_config() -> Config {
        let cli = Self::parse();
        match cli.command {
            Commands::Start {
                seed,
                idx,
                rpc,
                retries,
                ipfs_peers_config,
                rep_factor,
                keytable_file,
                latency,
            } => {
                let peers_config = PeersConfig::from_json(&ipfs_peers_config);

                Config::new(
                    seed,
                    idx,
                    rpc,
                    retries,
                    peers_config.ipfs_peers,
                    rep_factor,
                    keytable_file,
                    latency,
                )
            }
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Starts the pinning node
    Start {
        #[arg(short, long)]
        /// The seed phrase of the validator associated to the pinning node
        seed: String,
        #[arg(short, long)]
        /// The virtual node instance within all the nodes running in the same machine
        idx: u32,
        #[arg(short, long)]
        /// The endpoint of the chain rpc node
        rpc: String,
        /// The number of retries for a failed pinning operation
        #[arg(short, long)]
        retries: u8,
        /// The path of the json file containing the ipfs peers bounded to the pinning node
        #[arg(short, long)]
        ipfs_peers_config: String,
        /// The ring replication factor
        #[arg(short, long)]
        rep_factor: u32,
        /// The optional path to the file where the node keytable will be logged
        #[arg(short, long)]
        keytable_file: Option<String>,
        /// Whether to track latency
        #[arg(short, long)]
        latency: bool,
    },
}
