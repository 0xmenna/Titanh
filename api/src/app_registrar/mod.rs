use crate::{
	titanh::{
		self, app_registrar::events::AppCreated, capsules::calls::types::upload_capsule::App,
	},
	TitanhApi,
};
use anyhow::Result;
use sp_core::H256;

pub struct AppRegistrarApi<'a> {
	titanh: &'a TitanhApi,
}

impl<'a> From<&'a TitanhApi> for AppRegistrarApi<'a> {
	fn from(titanh: &'a TitanhApi) -> Self {
		AppRegistrarApi { titanh }
	}
}

impl AppRegistrarApi<'_> {
	/// Create a new app and wait for the transaction to be finalized
	pub async fn create_app(&self) -> Result<(App, H256)> {
		let app_tx = titanh::tx().app_registrar().create_app();
		let events = self.titanh.sign_and_submit_wait_finalized(&app_tx).await?;

		let app_event = events.find_first::<AppCreated>()?.unwrap();
		Ok((app_event.app_id, events.extrinsic_hash()))
	}
}
