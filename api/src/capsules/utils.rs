use super::{CapsulesApi, CapsulesConfig};
use anyhow::Result;

impl CapsulesApi<'_> {
	// ensures the api configuration is set
	pub fn ensure_config(&self) -> Result<&CapsulesConfig> {
		self.config
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("Capsules API configuration is not provided"))
	}
}
