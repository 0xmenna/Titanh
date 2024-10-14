use super::{CapsulesApi, CapsulesConfig};
use crate::{
    common_types::{Events, User},
    titanh::{
        self,
        capsules::calls::types::change_container_status::ContainerStatus,
        runtime_types::{pallet_capsules::pallet::Call, titanh_runtime::RuntimeCall},
    },
    DocumentApi,
};
use anyhow::Result;
use codec::Encode;
use sp_core::{Blake2Hasher, Hasher, H256};

pub struct ContainerApi<'a> {
    capsules: &'a CapsulesApi<'a>,
    config: &'a CapsulesConfig,
}

impl<'a> TryFrom<&'a CapsulesApi<'a>> for ContainerApi<'a> {
    type Error = anyhow::Error;
    fn try_from(capsules: &'a CapsulesApi) -> Result<Self, anyhow::Error> {
        if let Some(config) = &capsules.config {
            Ok(Self { capsules, config })
        } else {
            Err(anyhow::anyhow!("Capsules API not configured"))
        }
    }
}

impl<'a> ContainerApi<'a> {
    /// Create a container. It waits for the transaction finalization. This has strong consistency guarantees. Since this operation is expected to take place rarely for a single user, we encourage to use this method rather than making an on chain call with only the block inclusion guarantee.
    pub async fn create_container<Id: Encode>(
        &self,
        id: Id,
        other_owner: Option<User>,
    ) -> Result<H256> {
        let container_tx = titanh::tx().capsules().create_container(
            self.config.app,
            other_owner.map(|owner| owner.account()),
            id.encode(),
        );

        Ok(self
            .capsules
            .titanh
            .sign_and_submit_wait_finalized(&container_tx)
            .await?
            .extrinsic_hash())
    }

    /// Approve the ownership request of a container
    pub async fn approve_ownership<Id: Encode>(&self, id: Id) -> Result<Events> {
        let container_id = self.compute_id(id);
        let approval_tx = titanh::tx()
            .capsules()
            .approve_container_ownership(container_id);

        let events = self
            .capsules
            .titanh
            .sign_and_submit_wait_in_block(&approval_tx)
            .await?;

        Ok(events)
    }

    /// Share a container ownership
    pub async fn share_ownership<Id: Encode>(&self, id: Id, other_owner: User) -> Result<Events> {
        let container_id = self.compute_id(id);
        let share_tx = titanh::tx()
            .capsules()
            .share_container_ownership(container_id, other_owner.account());

        let events = self
            .capsules
            .titanh
            .sign_and_submit_wait_in_block(&share_tx)
            .await?;

        Ok(events)
    }

    /// Builds a call to attach a capsule to a container
    pub fn attach_capsule_call<Key: Encode, CapsuleId: Encode>(
        &self,
        container_id: H256,
        key: &Key,
        capsule_id: CapsuleId,
    ) -> RuntimeCall {
        let app_id = self.config.app;
        let capsule_id = self.capsules.compute_capsule_id(capsule_id, app_id);

        let call = RuntimeCall::Capsules(Call::container_put {
            container_id,
            key: key.encode(),
            capsule_id,
        });

        call
    }

    /// Builds a call to remove a capsule from a container
    pub fn remove_capsule_call<Id: Encode, Key: Encode>(&self, id: Id, key: &Key) -> RuntimeCall {
        let container_id = self.compute_id(id);

        let call = RuntimeCall::Capsules(Call::container_remove {
            container_id,
            key: key.encode(),
        });

        call
    }

    /// Changes the status of a container
    pub async fn set_status<Id: Encode>(&self, id: Id, status: ContainerStatus) -> Result<Events> {
        let container_id = self.compute_id(id);

        let status_tx = titanh::tx()
            .capsules()
            .change_container_status(container_id, status);

        let events = self
            .capsules
            .titanh
            .sign_and_submit_wait_in_block(&status_tx)
            .await?;

        Ok(events)
    }

    pub fn compute_id<Id: Encode>(&self, id: Id) -> H256 {
        let mut ids = Vec::new();

        ids.extend_from_slice(CONTAINER_ID_PREFIX);
        ids.extend_from_slice(&self.config.app.encode());
        ids.extend_from_slice(&id.encode());

        Blake2Hasher::hash(&ids[..])
    }

    pub fn document(&'a self) -> DocumentApi<'a> {
        DocumentApi::from(self)
    }
}

pub const CONTAINER_ID_PREFIX: &[u8] = b"cntnr";

pub mod document;
