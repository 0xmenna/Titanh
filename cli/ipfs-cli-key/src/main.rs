use clap::{Parser, Subcommand};
use sp_core::crypto::Pair as TraitPair;
use sp_core::ed25519;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ipfs-cli-key")]
#[command(
	about = "CLI tool to extract public key or sign messages using Ed25519 private key in PEM format"
)]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	/// Get the public key from the private key PEM file
	Public {
		/// Path to the PEM file (optional)
		#[arg(short, long)]
		pem: Option<String>,
	},
	/// Sign a message with the private key
	Sign {
		/// The message to sign
		message: String,
		/// Path to the PEM file (optional)
		#[arg(short, long)]
		pem: Option<String>,
	},
}

fn find_pem_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
	let current_dir = fs::read_dir(".")?;
	for entry in current_dir {
		let entry = entry?;
		if let Some(extension) = entry.path().extension() {
			if extension == "pem" {
				return Ok(entry.path());
			}
		}
	}
	Err("No PEM file found in the current directory".into())
}

fn load_keypair(pem_path: Option<String>) -> Result<ed25519::Pair, Box<dyn std::error::Error>> {
	let pem_path = match pem_path {
		Some(p) => PathBuf::from(p),
		None => find_pem_file()?,
	};

	let mut file = File::open(pem_path)?;
	let mut pem_contents = String::new();
	file.read_to_string(&mut pem_contents)?;

	let pem = pem::parse(pem_contents)?;
	let pair = ed25519::Pair::from_seed_slice(&pem.contents()[16..])?;
	Ok(pair)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let cli = Cli::parse();

	match &cli.command {
		Commands::Public { pem } => {
			let pair = load_keypair(pem.clone())?;
			let public_key = pair.public();
			println!("Public Key: 0x{}", hex::encode(&public_key));
		},
		Commands::Sign { message, pem } => {
			let pair = load_keypair(pem.clone())?;
			let signature = pair.sign(message.as_bytes());
			println!("Signature: 0x{}", hex::encode(signature));
		},
	}

	Ok(())
}
