use super::*;
use codec::Encode;
use common_types::BlockNumberFor;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use sp_core::{Get, Hasher};
use sp_std::vec;

use crate::Pallet as Capsules;
use pallet_app_registrar::{self as app_registrar, Config as AppRegistrarConfig, PermissionsApp};

fn create_app_from_caller<T: AppRegistrarConfig>(caller: T::AccountId) -> Result<(), &'static str> {
    app_registrar::Pallet::<T>::create_app(RawOrigin::Signed(caller.clone()).into())?;

    Ok(())
}

fn app_id<T: Config>() -> AppIdFor<T> {
    T::Permissions::current_app_id()
}

fn capsule<T: Config>() -> CapsuleUploadData<BlockNumberFor<T>> {
    let cid = b"QmfM2r8seH2GiRaC4esTjeraXEachRt8ZsSeGaWTPLyMoG".to_vec();
    let size = 13;
    let ending_retention_block: BlockNumberFor<T> = BlockNumberFor::<T>::from(1500u32);
    let followers_status = FollowersStatus::All;
    let encoded_metadata = vec![1, 2, 3];

    CapsuleUploadData {
        cid,
        size,
        ending_retention_block,
        followers_status,
        encoded_metadata,
    }
}

fn compute_capsule_id<T: Config>(app_id: AppIdFor<T>, metadata: Vec<u8>) -> T::Hash {
    let mut ids = Vec::new();

    ids.extend_from_slice(T::CapsuleIdPrefix::get());
    ids.extend_from_slice(&app_id.encode());
    ids.extend_from_slice(&metadata[..]);

    T::Hashing::hash(&ids[..])
}

benchmarks! {

    where_clause {
        where
            T: pallet_app_registrar::Config,
    }

    upload_capsule {
        let caller: T::AccountId = whitelisted_caller();

        // Create app
        create_app_from_caller::<T>(caller.clone())?;

        let app_id = app_id::<T>();
        let other_owner: Option<T::AccountId> = None;
        let capsule = capsule::<T>();
    }: _(RawOrigin::Signed(caller), app_id.clone(), other_owner, capsule)
    verify {
        // Verify that the capsule was uploaded correctly
        let capsule_id = compute_capsule_id::<T>(app_id, vec![1, 2, 3]);
        assert!(Capsules::<T>::capsules(capsule_id).is_some());
    }

    impl_benchmark_test_suite!(
        Capsules,
        crate::tests::new_test_ext(),
        crate::tests::Test
    );
}
