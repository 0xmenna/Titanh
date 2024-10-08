use codec::Encode;
use frame_support::{derive_impl, parameter_types};
use sp_core::{ConstU32, Hasher, H256};
use sp_std::vec;
// The testing primitives are very useful for avoiding having to work with signatures
// or public keys. `u64` is used as the `AccountId` and no `Signature`s are required.
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};
// Reexport crate as its pallet name for construct_runtime.
use crate as pallet_capsules;
use crate::*;
use frame_support::assert_ok;
use pallet_app_registrar::{self as app_registrar, CurrentAppId};

type Block = frame_system::mocking::MockBlock<Test>;

// For testing the pallet, we construct a mock runtime.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        AppRegistrar: app_registrar,
        Capsules: pallet_capsules,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type Nonce = u64;
    type Hash = H256;
    type RuntimeCall = RuntimeCall;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl app_registrar::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AppId = u32;
}

parameter_types! {
    pub CapsulePrefix: &'static [u8] = b"cpsl";
    pub ContainerPrefix: &'static [u8] = b"cntnr";
}

impl pallet_capsules::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type CapsuleIdPrefix = CapsulePrefix;
    type ContainerIdPrefix = ContainerPrefix;
    type MaxEncodedAppMetadata = ConstU32<1024>;
    type MaxOwners = ConstU32<32>;
    type StringLimit = ConstU32<32>;
    type Permissions = AppRegistrar;
    type RemoveItemsLimit = ConstU32<512>;
    // 1 hour, considering one block is 3 seconds
    type MinimumRetentionPeriod = ConstU32<50>;
    type CidLength = ConstU32<46>;
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = RuntimeGenesisConfig {
        // We use default for brevity, but you can configure as desired if needed.
        system: Default::default(),
    }
    .build_storage()
    .unwrap();
    t.into()
}

#[test]
fn upload_capsule_test() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(1);
        // First create app
        assert_ok!(AppRegistrar::create_app(origin.clone()));
        assert_eq!(CurrentAppId::<Test>::get(), 1);

        let capsule = CapsuleUploadData {
            // random IPFS cid
            cid: "QmfM2r8seH2GiRaC4esTjeraXEachRt8ZsSeGaWTPLyMoG"
                .as_bytes()
                .to_vec(),
            size: 13,
            ending_retention_block: 100,
            followers_status: FollowersStatus::All,
            encoded_metadata: vec![1, 2, 3],
        };

        assert_ok!(Capsules::upload_capsule(origin, 1, Some(2), capsule));

        let capsule_id = compute_capsule_id(1, vec![1, 2, 3]);
        assert!(Capsules::capsules(capsule_id).is_some());
    });
}

fn compute_capsule_id(app_id: u32, metadata: Vec<u8>) -> H256 {
    let mut ids = Vec::new();

    ids.extend_from_slice(CapsulePrefix::get());
    ids.extend_from_slice(&app_id.encode());
    ids.extend_from_slice(&metadata[..]);

    BlakeTwo256::hash(&ids[..])
}
