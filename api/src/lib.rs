use anyhow::Result;
use app_registrar::AppRegistrarApi;
use capsules::CapsulesApi;
use common::{
    titanh::{
        runtime_types::{frame_system::EventRecord, titanh_runtime::RuntimeEvent},
        utility::calls::types::batch_all::Calls as RuntimeCalls,
    },
    types::{
        BlockHash, BlockInfo, BlockNumber, ConsistencyLevel, Events, Rpc, Signer, SubstrateApi,
    },
};
use pinning_committee::PinningCommitteeApi;
use sp_core::H256;
use subxt::{storage::Address, tx::Payload, utils::Yes};

mod app_registrar;
mod builder;
mod capsules;
mod common;
mod pinning_committee;

// Export
pub use builder::TitanhApiBuilder;
pub use capsules::types as capsules_types;
pub use common::{titanh, types as common_types};
pub use pinning_committee::types as pinning_committee_types;

/// Titanh api
#[derive(Clone)]
pub struct TitanhApi {
    /// The Substrate api to query the chain storage
    pub substrate_api: SubstrateApi,
    /// The chain rpc methods
    pub rpc: Rpc,
    /// The singer of transactions
    pub signer: Option<Signer>,
}

impl TitanhApi {
    pub fn new(substrate_api: SubstrateApi, rpc: Rpc, signer: Option<Signer>) -> Self {
        TitanhApi {
            substrate_api,
            rpc,
            signer,
        }
    }

    /// Returns the app registrar api
    pub fn app_registrar(&self) -> AppRegistrarApi<'_> {
        AppRegistrarApi::from(self)
    }

    /// Returns the capsules api
    pub fn capsules(&self) -> CapsulesApi<'_> {
        CapsulesApi::from(self)
    }

    /// Returns the pinning committee api
    pub fn pinning_committee(&self) -> PinningCommitteeApi<'_> {
        PinningCommitteeApi::from(self)
    }

    /// Queries the chain's storage
    pub async fn query<'address, Addr>(
        &self,
        address: &'address Addr,
        at: Option<BlockHash>,
    ) -> Result<<Addr as Address>::Target>
    where
        Addr: Address<IsFetchable = Yes> + 'address,
    {
        let storage_client = self.substrate_api.storage();

        let storage = if let Some(block_hash) = at {
            storage_client.at(block_hash)
        } else {
            storage_client.at_latest().await?
        };

        // This returns an `Option<_>`, which will be
        // `None` if no value exists at the given address.
        let result = storage
            .fetch(address)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Value is not defined in storage"))?;
        Ok(result)
    }

    pub async fn runtime_events(
        &self,
        at: Option<BlockHash>,
    ) -> Result<Vec<EventRecord<RuntimeEvent, H256>>> {
        let events_query = titanh::storage().system().events();
        let runtime_events = self.query(&events_query, at).await?;

        Ok(runtime_events)
    }

    /// Returns the block hash of a n associated block number
    pub async fn block_hash(&self, block_number: BlockNumber) -> Result<BlockHash> {
        let block_hash = self
            .rpc
            .chain_get_block_hash(Some(block_number.into()))
            .await?;
        if let Some(hash) = block_hash {
            Ok(hash.into())
        } else {
            Err(anyhow::anyhow!(
                "Block hash not found for block number: {}",
                block_number
            ))
        }
    }

    pub async fn latest_finalized_block(&self) -> Result<BlockInfo> {
        let finalized_head = self.rpc.chain_get_finalized_head().await?;
        let block_num_query = titanh::storage().system().number();
        let number = self
            .query(&block_num_query, Some(finalized_head.into()))
            .await?;

        Ok(BlockInfo {
            number,
            hash: finalized_head.into(),
        })
    }

    fn ensure_signer(&self) -> Result<&Signer> {
        self.signer
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Signer is not set"))
    }

    /// Signs and submits a transaction. If it succeeds, it means the transaction is included in the transaction pool, not in a block.
    pub async fn sign_and_submit<Call: Payload>(&self, tx: &Call) -> Result<H256> {
        let signer = self.ensure_signer()?;
        let tx_hash = self
            .substrate_api
            .tx()
            .sign_and_submit_default(tx, signer)
            .await?;

        Ok(tx_hash)
    }

    /// Signs and submits a transaction. It waits for the transaction to be included in a block
    pub async fn sign_and_submit_wait_in_block<Call: Payload>(&self, tx: &Call) -> Result<Events> {
        let signer = self.ensure_signer()?;
        let mut tx_progress = self
            .substrate_api
            .tx()
            .sign_and_submit_then_watch_default(tx, signer)
            .await?;

        while let Some(block_status) = tx_progress.next().await {
            let status = block_status?;
            if let Some(in_block) = status.as_in_block() {
                let events = in_block.wait_for_success().await?;
                return Ok(events);
            }
        }

        Err(anyhow::anyhow!("Transaction failed"))
    }

    /// Signs and submits a transaction. It waits until the transaction is finalized.
    pub async fn sign_and_submit_wait_finalized<Call: Payload>(&self, tx: &Call) -> Result<Events> {
        let signer = self.ensure_signer()?;

        // Submit the extrinisc, and wait for it to be successful and in a finalized block.
        // We get back the extrinsic events if all is well.
        let events = self
            .substrate_api
            .tx()
            .sign_and_submit_then_watch_default(tx, signer)
            .await?
            .wait_for_finalized_success()
            .await?;

        Ok(events)
    }

    pub async fn sign_and_submit_tx_with_level<Call: Payload>(
        &self,
        tx: &Call,
        level: ConsistencyLevel,
    ) -> Result<H256> {
        let tx_hash = match level {
            // Just include the transaction in the transaction pool
            ConsistencyLevel::Low => self.sign_and_submit(tx).await?,
            // Wait for block inclusion
            ConsistencyLevel::Medium => {
                let events = self.sign_and_submit_wait_in_block(tx).await?;
                events.extrinsic_hash()
            }
            // Wait for block finalization
            ConsistencyLevel::High => {
                let events = self.sign_and_submit_wait_finalized(tx).await?;
                events.extrinsic_hash()
            }
        };

        Ok(tx_hash)
    }

    /// Signs and submits a batch of transactions (all or nothing). It waits until the transaction is finalized.
    pub async fn sing_and_submit_batch(
        &self,
        calls: RuntimeCalls,
        level: ConsistencyLevel,
    ) -> Result<H256> {
        let batch_tx = titanh::tx().utility().batch_all(calls);

        let tx_hash = match level {
            ConsistencyLevel::Low => self.sign_and_submit(&batch_tx).await?,
            ConsistencyLevel::Medium => self
                .sign_and_submit_wait_in_block(&batch_tx)
                .await?
                .extrinsic_hash(),
            ConsistencyLevel::High => self
                .sign_and_submit_wait_finalized(&batch_tx)
                .await?
                .extrinsic_hash(),
        };

        Ok(tx_hash)
    }
}
