use crate::utils::Config;
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce}; // Or `Aes128Gcm`
use codec::{Decode, Encode};
use rand::{thread_rng, Rng};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;

#[derive(Deserialize, Encode, Decode, Clone)]
pub struct CertificateId {
	pub university_id: String,
	pub student_id: String,
}

#[derive(Encode, Decode, Clone)]
pub struct Certificate {
	pub student_name: String,
	pub degree_program: String,
	pub graduation_year: String,
	pub grade: String,
	/// The encrypted certificate
	pub data: Vec<u8>,
}

impl Certificate {
	/// Build the certificate from the configuration file.
	pub fn from_config(config: &Config) -> (CertificateId, Certificate) {
		let cert_config = config.certificate.clone();
		// Read the file content
		let mut file = File::open(&cert_config.path).expect("Failed to open the certificate file");
		let mut certificate = Vec::new();
		file.read_to_end(&mut certificate).expect("Failed to read the certificate file");

		(
			cert_config.id,
			Certificate {
				student_name: cert_config.student_name,
				degree_program: cert_config.degree_program,
				graduation_year: cert_config.graduation_year,
				grade: cert_config.grade,
				data: certificate,
			},
		)
	}

	/// Generate a random 32-byte key for certificate encryption.
	pub fn gen_cert_key() -> [u8; 32] {
		thread_rng().gen::<[u8; 32]>()
	}

	/// Encrypts the certificate using AES-GCM.
	pub fn encrypt(self, key: &[u8; 32]) -> Self {
		// Generate a random 12-byte nonce (GCM standard)
		let nonce = rand::thread_rng().gen::<[u8; 12]>();

		// Initialize AES-256-GCM with the provided key
		let cipher = Aes256Gcm::new_from_slice(key).unwrap();

		// Encrypt the file content, with the nonce required for decryption
		let ciphertext = cipher
			.encrypt(Nonce::from_slice(&nonce), self.data.as_ref())
			.expect("encryption failure!");

		// Combine nonce and ciphertext (nonce is required for decryption)
		let mut encrypted_data = nonce.to_vec();
		encrypted_data.extend(ciphertext);

		Self { data: encrypted_data, ..self }
	}
}
