use crate::TitanhApi;

pub struct PinningCommitteeApi<'a> {
	titanh: &'a TitanhApi,
}

impl<'a> From<&'a TitanhApi> for PinningCommitteeApi<'a> {
	fn from(titanh: &'a TitanhApi) -> Self {
		PinningCommitteeApi::new(titanh)
	}
}

impl<'a> PinningCommitteeApi<'a> {
	pub fn new(titanh: &'a TitanhApi) -> Self {
		PinningCommitteeApi { titanh }
	}
}
