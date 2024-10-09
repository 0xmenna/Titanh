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
    /// Acquire metrics for put, get and batch_put operations
    BytesRange {
        /// The start of the range in bytes: K for kilobytes, M for megabytes, G for gigabytes
        #[arg(short, long)]
        start: BytesSize,
        /// The end of the range in bytes: K for kilobytes, M for megabytes, G for gigabytes
        #[arg(short, long)]
        end: BytesSize,
        /// The step of bytes to incremenent for each sample
        #[arg(short, long)]
        step: BytesSize,
    },
}
