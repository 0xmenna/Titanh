use super::crypto::IpfsPair;
use crate::titanh::{
	pinning_committee::Call,
	runtime_types::{
		pallet_pinning_committee::types::RegistrationMessage, titanh_runtime::RuntimeCall,
	},
};

pub const REGISTRATION_MESSAGE: &[u8] = b"Pinning node registration";

pub fn build_rep_factor_call(rep_factor: u32) -> RuntimeCall {
	RuntimeCall::PinningCommittee(Call::set_content_replication_factor { factor: rep_factor })
}

pub fn build_ipfs_replicas_call(ipfs_replicas: u32) -> RuntimeCall {
	RuntimeCall::PinningCommittee(Call::set_ipfs_replicas { ipfs_replicas })
}

pub fn build_pinning_nodes_call(nodes_num: u32) -> RuntimeCall {
	RuntimeCall::PinningCommittee(Call::set_pinning_nodes_per_validator {
		pinning_nodes: nodes_num,
	})
}

pub fn build_registration_message_call(pair: &IpfsPair) -> RuntimeCall {
	let registration = RegistrationMessage {
		key: pair.public().try_into().unwrap(),
		signature: pair.sign(REGISTRATION_MESSAGE).try_into().unwrap(),
	};

	RuntimeCall::PinningCommittee(Call::register_ipfs_node { registration })
}
