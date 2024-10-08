use clap::{Parser, Subcommand};

use crate::types::BytesSize;

#[derive(Parser)]
#[command(name = "metrics")]
#[command(about = "CLI tool to acquire metrics of interest for the datastore")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Acquire metrics for put and get operations
    BytesRange {
        /// The start of the range in bytes: K for kilobytes, M for megabytes, G for gigabytes
        #[arg(short, long)]
        start: BytesSize,
        /// The end of the range in bytes: K for kilobytes, M for megabytes, G for gigabytes
        #[arg(short, long)]
        end: BytesSize,
        /// The number of samples to acquire
        #[arg(short, long)]
        samples: u64,
    },
}
