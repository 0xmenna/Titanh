use super::chain::titanh::runtime_types::bounded_collections::bounded_vec::BoundedVec;

#[derive(Clone)]
pub struct Cid(String);

impl TryFrom<BoundedVec<u8>> for Cid {
	type Error = anyhow::Error;

	fn try_from(cid: BoundedVec<u8>) -> Result<Self, Self::Error> {
		let cid = std::str::from_utf8(&cid.0)?;
		Ok(Cid(cid.to_string()))
	}
}

impl AsRef<str> for Cid {
	fn as_ref(&self) -> &str {
		&self.0
	}
}
