use crate::{
    common_types::ConsistencyLevel,
    titanh::{
        self,
        runtime_types::{
            bounded_collections::bounded_vec::BoundedVec, primitives::common_types::BoundedString,
        },
        utility::calls::types::batch::Calls,
    },
};

use super::ContainerApi;
use anyhow::Result;
use codec::{Decode, Encode};
use sp_core::{Blake2Hasher, Hasher, H256};

pub struct DocumentApi<'a> {
    container_api: &'a ContainerApi<'a>,
}

impl<'a> From<&'a ContainerApi<'a>> for DocumentApi<'a> {
    fn from(container_api: &'a ContainerApi<'a>) -> Self {
        DocumentApi { container_api }
    }
}

impl DocumentApi<'_> {
    pub fn document_from_id<Id: Encode>(&self, id: Id) -> Document {
        let id = self.container_api.compute_id(id);
        Document {
            id,
            api: self.container_api,
        }
    }

    pub async fn create_document<Id: Encode>(&self, id: Id) -> Result<Document> {
        let _ = self.container_api.create_container(&id, None).await?;
        let id = self.container_api.compute_id(id);
        Ok(Document {
            id,
            api: self.container_api,
        })
    }
}

type DocumentId = H256;
pub struct Document<'a> {
    id: DocumentId,
    api: &'a ContainerApi<'a>,
}

impl Document<'_> {
    /// Insert a field identified by a key into the document, waiting for the transaction to be included in a block.
    pub async fn insert<Key, Value>(&self, field_key: Key, value: Value) -> Result<H256>
    where
        Key: Encode,
        Value: Encode,
    {
        self.insert_with_level(field_key, value, ConsistencyLevel::default())
            .await
    }

    /// Insert a field identified by a key into the document, waiting for the tx to be finalized
    pub async fn insert_wait_finalization<Key, Value>(
        &self,
        field_key: Key,
        value: Value,
    ) -> Result<H256>
    where
        Key: Encode,
        Value: Encode,
    {
        self.insert_with_level(field_key, value, ConsistencyLevel::High)
            .await
    }

    /// Insert a field identified by a key into the document, waiting for the transaction to be included in the transaction pool
    pub async fn insert_async<Key, Value>(&self, field_key: Key, value: Value) -> Result<H256>
    where
        Key: Encode,
        Value: Encode,
    {
        self.insert_with_level(field_key, value, ConsistencyLevel::Low)
            .await
    }

    pub async fn insert_with_level<Key, Value>(
        &self,
        field_key: Key,
        value: Value,
        level: ConsistencyLevel,
    ) -> Result<H256>
    where
        Key: Encode,
        Value: Encode,
    {
        let mut calls = Calls::new();

        let capsule_id = self.compute_capsule_id(&field_key);

        // First, upload the capsule to ipfs. We get the corresponding runtime call associated to the upload
        let runtime_call = self
            .api
            .capsules
            .upload_capsule_to_ifps(&capsule_id, value)
            .await?;
        calls.push(runtime_call);

        let container_attach_call = self.api.attach_capsule_call(self.id, field_key, capsule_id);
        calls.push(container_attach_call);

        let tx_hash = self
            .api
            .capsules
            .titanh
            .sing_and_submit_batch(calls, level)
            .await?;

        Ok(tx_hash)
    }

    /// Reads a document entry from a latest block
    pub async fn read<Key, Value>(&self, field_key: Key) -> Result<Value>
    where
        Key: Encode,
        Value: Decode,
    {
        self.read_with_opts(field_key, false).await
    }

    /// Reads a document entry from a finalized state
    pub async fn read_finalized<Key, Value>(&self, field_key: Key) -> Result<Value>
    where
        Key: Encode,
        Value: Decode,
    {
        self.read_with_opts(field_key, true).await
    }

    pub async fn read_with_opts<Key, Value>(
        &self,
        field_key: Key,
        from_finalized_state: bool,
    ) -> Result<Value>
    where
        Key: Encode,
        Value: Decode,
    {
        let field_key = field_key.encode();

        let key = BoundedString(BoundedVec(field_key));
        let query_container_capsule = titanh::storage().capsules().container(self.id, key);

        let at = if from_finalized_state {
            Some(
                self.api
                    .capsules
                    .titanh
                    .latest_finalized_block()
                    .await?
                    .hash,
            )
        } else {
            None
        };

        let capsule_id = self
            .api
            .capsules
            .titanh
            .query(&query_container_capsule, at)
            .await?;

        let value = self
            .api
            .capsules
            .read_capsule(capsule_id, from_finalized_state)
            .await?;

        Ok(value)
    }

    /// Removes a document entry (without unlinking the underlining capsule) waiting for the transaction to be included in a block
    pub async fn remove<Key: Encode>(&self, field_key: Key) -> Result<H256> {
        self.remove_with_level(field_key, ConsistencyLevel::default())
            .await
    }

    /// Removes a document entry (without unlinking the underlining capsule) waiting for the tx to be finalized
    pub async fn remove_wait_finalization<Key: Encode>(&self, field_key: Key) -> Result<H256> {
        self.remove_with_level(field_key, ConsistencyLevel::High)
            .await
    }

    /// Removes a document entry (without unlinking the underlining capsule) waiting for the transaction to be included in the transaction pool
    pub async fn remove_async<Key: Encode>(&self, field_key: Key) -> Result<H256> {
        self.remove_with_level(field_key, ConsistencyLevel::Low)
            .await
    }

    /// Removes a document entry (without unlinking the underlining capsule)
    pub async fn remove_with_level<Key: Encode>(
        &self,
        field_key: Key,
        level: ConsistencyLevel,
    ) -> Result<H256> {
        let rm_container_tx = titanh::tx()
            .capsules()
            .container_remove(self.id, field_key.encode());

        self.api
            .capsules
            .titanh
            .sign_and_submit_tx_with_level(&rm_container_tx, level)
            .await
    }

    pub fn id(&self) -> &DocumentId {
        &self.id
    }

    pub fn compute_capsule_id<Key: Encode>(&self, key: Key) -> H256 {
        let mut capsule_id_material = Vec::new();

        capsule_id_material.extend_from_slice(&self.id.encode());
        capsule_id_material.extend_from_slice(&key.encode());

        let id = Blake2Hasher::hash(&capsule_id_material[..]);

        let capsule_id = self
            .api
            .capsules
            .compute_capsule_id(id, self.api.config.app);

        capsule_id
    }
}
