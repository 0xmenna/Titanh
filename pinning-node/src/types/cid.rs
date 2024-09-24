use codec::{Decode, Encode};

#[derive(Clone, Encode, Decode)]
pub struct Cid(String);

impl TryFrom<Vec<u8>> for Cid {
	type Error = anyhow::Error;

	fn try_from(cid: Vec<u8>) -> Result<Self, Self::Error> {
		let cid = std::str::from_utf8(&cid)?;
		Ok(Cid(cid.to_string()))
	}
}

impl AsRef<str> for Cid {
	fn as_ref(&self) -> &str {
		&self.0
	}
}
