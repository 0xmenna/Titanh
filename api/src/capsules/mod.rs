use crate::{
    common_types::{BlockNumber, ConsistencyLevel, Events, User},
    titanh::{
        self,
        capsules::calls::types::upload_capsule::App,
        runtime_types::{
            pallet_capsules::{
                capsule::types::CapsuleUploadData, pallet::Call, types::FollowersStatus,
            },
            titanh_runtime::RuntimeCall,
        },
        utility::calls::types::batch_all::Calls,
    },
    TitanhApi,
};
use anyhow::{Ok, Result};
use codec::{Decode, Encode};
use futures::TryStreamExt;
use ipfs_api_backend_hyper::{request::Add, IpfsApi, IpfsClient, TryFromUri};
use sp_core::H256;
use std::io::Cursor;
use types::{CapsulesBatch, GetCapsuleOpts, PutCapsuleOpts, UpdateCapsuleOpts};
use utils::convert_bounded_str;

pub struct CapsulesConfig {
    ipfs: IpfsClient,
    app: App,
}

pub struct CapsulesApi<'a> {
    titanh: &'a TitanhApi,
    config: Option<CapsulesConfig>,
}

impl<'a> From<&'a TitanhApi> for CapsulesApi<'a> {
    fn from(titanh: &'a TitanhApi) -> Self {
        CapsulesApi {
            titanh,
            config: None,
        }
    }
}

impl<'a> CapsulesApi<'a> {
    /// Provides the IPFS RPC URL and the app id as configuration
    pub fn config(self, ipfs_rpc_url: &str, app: App) -> Result<Self> {
        let ipfs = IpfsClient::from_str(ipfs_rpc_url)?;
        Ok(Self {
            config: Some(CapsulesConfig { ipfs, app }),
            ..self
        })
    }

    /// Put a new object identified by `id` to IPFS and add the metadata to the chain with default options. The transaction is waited for block inclusion.
    pub async fn put<Id, Value>(&self, id: Id, data: Value) -> Result<H256>
    where
        Id: Encode,
        Value: Encode,
    {
        let tx_hash = self
            .put_with_options(id, data, PutCapsuleOpts::default())
            .await?;
        Ok(tx_hash)
    }

    /// Put a new object identified by `id` to IPFS and add the metadata to the chain. The transaction is async by means of not waiting for block inclusion, but just an inclusion in the transaction pool.
    pub async fn put_async<Id, Value>(&self, id: Id, data: Value) -> Result<H256>
    where
        Id: Encode,
        Value: Encode,
    {
        let mut opts = PutCapsuleOpts::default();
        opts.level = ConsistencyLevel::Low;

        let tx_hash = self.put_with_options(id, data, opts).await?;
        Ok(tx_hash)
    }

    /// Put a new object identified by `id` to IPFS and add the metadata to the chain, waiting for the transaction to be finalized
    pub async fn put_wait_finalized<Id, Value>(&self, id: Id, data: Value) -> Result<H256>
    where
        Id: Encode,
        Value: Encode,
    {
        let mut opts = PutCapsuleOpts::default();
        opts.level = ConsistencyLevel::High;

        let tx_hash = self.put_with_options(id, data, opts).await?;

        Ok(tx_hash)
    }

    /// Put a new object identified by `id` to IPFS and add the metadata to the chain, given the options
    pub async fn put_with_options<Id, Data>(
        &self,
        id: Id,
        data: Data,
        options: PutCapsuleOpts,
    ) -> Result<H256>
    where
        Id: Encode,
        Data: Encode,
    {
        // Ensure the configuration is set
        let config = self.ensure_config()?;

        let (cid, size) = self.upload_to_ipfs(data).await?;
        let (retention_blocks, consistency_level) = options.unwrap_fields_or_default();
        let ending_retention_block =
            self.titanh.latest_finalized_block().await?.number + retention_blocks;

        // Build the capsule (by default does not allow followers)
        let capsule = CapsuleUploadData {
            cid,
            size,
            ending_retention_block,
            followers_status: FollowersStatus::None,
            encoded_metadata: id.encode(),
        };

        let upload_tx = titanh::tx()
            .capsules()
            .upload_capsule(config.app, None, capsule);

        let tx_hash = self
            .titanh
            .sign_and_submit_tx_with_level(&upload_tx, consistency_level)
            .await?;

        Ok(tx_hash)
    }

    /// Put a batch of capsules to IPFS and add the metadata to the chain, waiting for transaction pool inclusion
    pub async fn put_batch_async<Id, Value>(&self, batch: CapsulesBatch<Id, Value>) -> Result<H256>
    where
        Id: Encode,
        Value: Encode,
    {
        let mut opts = PutCapsuleOpts::default();
        opts.level = ConsistencyLevel::Low;

        let tx_hash = self.put_batch_with_options(batch, opts).await?;
        Ok(tx_hash)
    }

    /// Put a batch of capsules to IPFS and add the metadata to the chain, waiting for block finalization
    pub async fn put_batch_wait_finalized<Id, Value>(
        &self,
        batch: CapsulesBatch<Id, Value>,
    ) -> Result<H256>
    where
        Id: Encode,
        Value: Encode,
    {
        let mut opts = PutCapsuleOpts::default();
        opts.level = ConsistencyLevel::High;

        let tx_hash = self.put_batch_with_options(batch, opts).await?;
        Ok(tx_hash)
    }

    /// Put a batch of capsules to IPFS and add the metadata to the chain, waiting for block inclusion
    pub async fn put_batch<Id, Value>(&self, batch: CapsulesBatch<Id, Value>) -> Result<H256>
    where
        Id: Encode,
        Value: Encode,
    {
        let tx_hash = self
            .put_batch_with_options(batch, PutCapsuleOpts::default())
            .await?;
        Ok(tx_hash)
    }

    pub async fn put_batch_with_options<Id, Value>(
        &self,
        batch: CapsulesBatch<Id, Value>,
        options: PutCapsuleOpts,
    ) -> Result<H256>
    where
        Id: Encode,
        Value: Encode,
    {
        let mut calls = Calls::new();
        let finalized_block = self.titanh.latest_finalized_block().await?.number;
        for (id, value) in batch {
            let runtime_call = self
                .upload_capsule_to_ifps(id, value, &options, finalized_block)
                .await?;
            calls.push(runtime_call);
        }

        let tx_hash = self
            .titanh
            .sign_and_submit_batch(calls, options.level)
            .await?;
        Ok(tx_hash)
    }

    /// Removes a capsules. Waits for block inclusion
    pub async fn remove<Id: Encode>(&self, id: Id) -> Result<H256> {
        let tx_hash = self
            .remove_with_level(id, ConsistencyLevel::default())
            .await?;
        Ok(tx_hash)
    }

    /// Removes a capsules. Wait for block finalization
    pub async fn remove_wait_finalized<Id: Encode>(&self, id: Id) -> Result<H256> {
        let tx_hash = self.remove_with_level(id, ConsistencyLevel::High).await?;
        Ok(tx_hash)
    }

    /// Removes a capsules. Only waits for transaction pool inclusion
    pub async fn remove_async<Id: Encode>(&self, id: Id) -> Result<H256> {
        let tx_hash = self.remove_with_level(id, ConsistencyLevel::Low).await?;
        Ok(tx_hash)
    }

    pub async fn remove_with_level<Id: Encode>(
        &self,
        id: Id,
        level: ConsistencyLevel,
    ) -> Result<H256> {
        let config = self.ensure_config()?;
        let capsule_id = self.compute_capsule_id(id, config.app);

        let remove_tx = titanh::tx().capsules().start_destroy_capsule(capsule_id);

        let tx_hash = self
            .titanh
            .sign_and_submit_tx_with_level(&remove_tx, level)
            .await?;

        Ok(tx_hash)
    }

    /// Reads a value from the latest block, not yet finalized
    pub async fn get<Id: Encode, Value: Decode>(&self, id: Id) -> Result<Value> {
        let opts = GetCapsuleOpts::default();
        let value = self.get_with_options(id, opts).await?;

        Ok(value)
    }

    /// Reads a value from a finalized block
    pub async fn get_finalized<Id: Encode, Value: Decode>(&self, id: Id) -> Result<Value> {
        let opts = GetCapsuleOpts {
            from_finalized_state: true,
        };

        let value = self.get_with_options(id, opts).await?;

        Ok(value)
    }

    pub async fn get_with_options<Id: Encode, Value: Decode>(
        &self,
        id: Id,
        opts: GetCapsuleOpts,
    ) -> Result<Value> {
        // Ensure the configuration is set
        let config = self.ensure_config()?;

        let capsule_id = self.compute_capsule_id(id, config.app);
        let value = self
            .read_capsule(capsule_id, opts.from_finalized_state)
            .await?;

        Ok(value)
    }

    /// Updates the content of a capsule. Waits for block inclusion
    pub async fn update<Id: Encode, Value: Encode>(&self, id: Id, data: Value) -> Result<H256> {
        let tx_hash = self
            .update_with_options(id, data, UpdateCapsuleOpts::default())
            .await?;
        Ok(tx_hash)
    }

    /// Updates the content of a capsule. Does not wait for block inclusion
    pub async fn update_async<Id: Encode, Value: Encode>(
        &self,
        id: Id,
        data: Value,
    ) -> Result<H256> {
        let mut opts = UpdateCapsuleOpts::default();
        opts.level = ConsistencyLevel::Low;

        let tx_hash = self.update_with_options(id, data, opts).await?;
        Ok(tx_hash)
    }

    /// Updates the content of a capsule. Waits for block finalization
    pub async fn update_wait_finalized<Id: Encode, Value: Encode>(
        &self,
        id: Id,
        data: Value,
    ) -> Result<H256> {
        let mut opts = UpdateCapsuleOpts::default();
        opts.level = ConsistencyLevel::High;

        let tx_hash = self.update_with_options(id, data, opts).await?;
        Ok(tx_hash)
    }

    pub async fn update_with_options<Id: Encode, Value: Encode>(
        &self,
        id: Id,
        data: Value,
        opts: UpdateCapsuleOpts,
    ) -> Result<H256> {
        let config = self.ensure_config()?;

        let capsule_id = self.compute_capsule_id(id, config.app);
        let (cid, size) = self.upload_to_ipfs(data).await?;

        let update_tx = titanh::tx()
            .capsules()
            .update_capsule_content(capsule_id, cid, size);

        let tx_hash = self
            .titanh
            .sign_and_submit_tx_with_level(&update_tx, opts.level)
            .await?;

        Ok(tx_hash)
    }

    /// Shares the ownership of a capsule with another user
    pub async fn share_ownership<Id: Encode>(&self, id: Id, who: User) -> Result<Events> {
        let config = self.ensure_config()?;
        let capsule_id = self.compute_capsule_id(id, config.app);

        let tx_share = titanh::tx()
            .capsules()
            .share_capsule_ownership(capsule_id, who.account());

        let events = self.titanh.sign_and_submit_wait_in_block(&tx_share).await?;
        Ok(events)
    }

    /// Approves the ownership request of a capsule
    pub async fn approve_ownership<Id: Encode>(&self, id: Id) -> Result<Events> {
        let config = self.ensure_config()?;
        let capsule_id = self.compute_capsule_id(id, config.app);

        let tx_share = titanh::tx()
            .capsules()
            .approve_capsule_ownership(capsule_id);

        let events = self.titanh.sign_and_submit_wait_in_block(&tx_share).await?;
        Ok(events)
    }

    /// Set the followers status of a capsule
    pub async fn set_followers_status<Id: Encode>(
        &self,
        id: Id,
        status: FollowersStatus,
    ) -> Result<Events> {
        let config = self.ensure_config()?;
        let capsule_id = self.compute_capsule_id(id, config.app);

        let tx_followers = titanh::tx()
            .capsules()
            .set_capsule_followers_status(capsule_id, status);

        let events = self
            .titanh
            .sign_and_submit_wait_in_block(&tx_followers)
            .await?;
        Ok(events)
    }

    /// Follow a capsule
    pub async fn follow<Id: Encode>(&self, id: Id) -> Result<Events> {
        let config = self.ensure_config()?;
        let capsule_id = self.compute_capsule_id(id, config.app);

        let tx_follow = titanh::tx().capsules().follow_capsule(capsule_id);

        let events = self
            .titanh
            .sign_and_submit_wait_in_block(&tx_follow)
            .await?;
        Ok(events)
    }

    /// Add a priviledged follower
    pub async fn add_priviledged_follower<Id: Encode>(
        &self,
        id: Id,
        follower: User,
    ) -> Result<Events> {
        let config = self.ensure_config()?;
        let capsule_id = self.compute_capsule_id(id, config.app);

        let tx_add = titanh::tx()
            .capsules()
            .add_priviledged_follower(capsule_id, follower.account());

        let events = self.titanh.sign_and_submit_wait_in_block(&tx_add).await?;
        Ok(events)
    }

    /// Approve a priviledged follower request
    pub async fn approve_priviledged<Id: Encode>(&self, id: Id) -> Result<Events> {
        let config = self.ensure_config()?;
        let capsule_id = self.compute_capsule_id(id, config.app);

        let tx_approve = titanh::tx()
            .capsules()
            .approve_privileged_follow(capsule_id);

        let events = self
            .titanh
            .sign_and_submit_wait_in_block(&tx_approve)
            .await?;
        Ok(events)
    }

    pub async fn upload_capsule_to_ifps<Id, Data>(
        &self,
        id: Id,
        data: Data,
        opts: &PutCapsuleOpts,
        finalized_block: BlockNumber,
    ) -> Result<RuntimeCall>
    where
        Id: Encode,
        Data: Encode,
    {
        let config = self.ensure_config()?;

        let (cid, size) = self.upload_to_ipfs(data).await?;
        let (retention_blocks, _) = opts.unwrap_fields_or_default();

        let ending_retention_block = finalized_block + retention_blocks;

        // Build the capsule
        let capsule = CapsuleUploadData {
            cid,
            size,
            ending_retention_block,
            followers_status: FollowersStatus::None,
            encoded_metadata: id.encode(),
        };

        let call = RuntimeCall::Capsules(Call::upload_capsule {
            app: config.app,
            other_owner: None,
            capsule: capsule,
        });

        Ok(call)
    }

    async fn upload_to_ipfs<Data: Encode>(&self, data: Data) -> Result<(Vec<u8>, u128)> {
        let config = self.ensure_config()?;

        let data = Cursor::new(data.encode());

        // Do not pin the data
        let mut add_opts = Add::default();
        add_opts.pin = Some(false);
        // Add the data to IPFS
        let ipfs_res = config.ipfs.add_with_options(data, add_opts).await?;

        let cid = ipfs_res.hash.as_bytes().to_vec();
        let size: u128 = ipfs_res
            .size
            .parse()
            .expect("Content size is expected to be a valid number");

        Ok((cid, size))
    }

    pub async fn read_capsule<Value: Decode>(
        &self,
        capsule_id: H256,
        from_finalized_state: bool,
    ) -> Result<Value> {
        let config = self.ensure_config()?;
        let capsule_query = titanh::storage().capsules().capsules(capsule_id);

        let at = if from_finalized_state {
            Some(self.titanh.latest_finalized_block().await?)
        } else {
            None
        }
        .map(|block| block.hash);

        let capsule = self.titanh.query(&capsule_query, at).await?;
        let cid = convert_bounded_str(capsule.cid)?;

        let response = config
            .ipfs
            .cat(&cid)
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
            .await
            .map_err(|_| anyhow::anyhow!("error reading full file"))?;

        let value = Value::decode(&mut &response[..])?;

        Ok(value)
    }
}

pub mod container;
pub mod types;
pub mod utils;
