// This is a use case example on how to use the Titanh API to create a document, by insert a student certificate into it.

use anyhow::Result;
use clap::{Parser, Subcommand};
use hex;
use titan_api::TitanhApiBuilder;
use types::Certificate;
use utils::Config;

mod types;
mod utils;

const APP: u32 = 1;

#[derive(Parser)]
#[command(name = "uni-certificate")]
#[command(about = "CLI tool to manage university certificates")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Write a document
    WriteDocument {},
    /// Read a document
    ReadDocument {
        /// The key to decrypt the certificate (hex string)
        #[arg(short, long)]
        key: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::WriteDocument {} => {
            write_document().await?;
        }
        Commands::ReadDocument { key } => {
            read_student_certificate(key).await?;
        }
    }

    Ok(())
}

async fn write_document() -> Result<()> {
    // Read app configuration
    let config = Config::from_json();
    // Build the Titanh API
    let api = TitanhApiBuilder::rpc(&config.chain_rpc)
        .seed(&config.seed)
        .build()
        .await?;

    // Get the capsules API and configure it with the IPFS RPC URL and the app ID
    let capsules = api.capsules().config(&config.ipfs_rpc, APP)?;

    // Application logic
    let key = Certificate::gen_cert_key();
    let (_, certificate) = Certificate::from_config(&config);
    let certificate = certificate.encrypt(&key);

    // Create document API
    let container_api = capsules.container()?;
    let doc_api = container_api.document();

    println!("Creating document...");
    // Create document
    let doc = doc_api
        .create_document("computer_engineering_degrees")
        .await?;

    // Insert student certificate into the document
    let student_name = config.certificate.student_name;

    let tx_hash = doc.insert_async(student_name, certificate).await?;

    // Print results
    println!(
        "Certificate uploaded with tx hash: 0x{}",
        hex::encode(tx_hash.as_bytes())
    );
    println!("Certificate encryption key: 0x{}", hex::encode(key));

    Ok(())
}

pub async fn read_student_certificate(key_hex: String) -> Result<()> {
    // Read app configuration
    let config = Config::from_json();
    // Build the Titanh API
    let api = TitanhApiBuilder::rpc(&config.chain_rpc)
        .seed(&config.seed)
        .build()
        .await?;

    // Get the capsules API and configure it with the IPFS RPC URL and the app ID
    let capsules = api.capsules().config(&config.ipfs_rpc, APP)?;

    // Create document API
    let container_api = capsules.container()?;
    let doc_api = container_api.document();

    let doc = doc_api.document_from_id("computer_engineering_degrees");

    let cert: Certificate = doc.read(config.certificate.student_name).await?;

    cert.decrypt_to_file(key_hex, "data/read-certificate.jpg");

    Ok(())
}
