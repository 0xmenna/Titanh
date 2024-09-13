/// This module provides some utilities to work with the Substrate storage.
/// In particular, to build storage keys
use codec::Encode;
use frame_support::{Blake2_128Concat, StorageHasher, Twox128, Twox64Concat};
use sp_core::storage::StorageKey;

#[derive(Default)]
pub struct StorageUtilReady;
/// The module name that contains the storage element
pub struct ModuleName(String);
/// The storage name of the module
pub struct StorageName(String);
/// The prefix of the storage key
pub struct StoragePrefix {
	module: ModuleName,
	storage: StorageName,
}

pub type StorageItemsKeyHash = Vec<u8>;
/// The prefix and suffix of a storage key
pub struct StorageKeyData {
	prefix: StoragePrefix,
	suffix: Vec<StorageItemsKeyHash>,
}

/// A storage item key within its hashing algorithm
pub enum StorageItemKey<T> {
	Blake2_128Concat(T),
	Twox64Concat(T),
}

#[derive(Default)]
pub struct StorageKeyBuilder<T>(T);

impl StorageKeyBuilder<StorageUtilReady> {
	pub fn module_name(self, name: &str) -> StorageKeyBuilder<ModuleName> {
		StorageKeyBuilder(ModuleName(name.to_string()))
	}
}

impl StorageKeyBuilder<ModuleName> {
	pub fn storage_name(self, name: &str) -> StorageKeyBuilder<StoragePrefix> {
		StorageKeyBuilder(StoragePrefix { module: self.0, storage: StorageName(name.to_string()) })
	}
}

impl StorageKeyBuilder<StoragePrefix> {
	pub fn create_storage_items(self) -> StorageKeyBuilder<StorageKeyData> {
		StorageKeyBuilder(StorageKeyData { prefix: self.0, suffix: Vec::new() })
	}
}

impl StorageKeyBuilder<StorageKeyData> {
	pub fn push_item_key<T: Encode>(&mut self, key: StorageItemKey<T>) {
		match key {
			StorageItemKey::Blake2_128Concat(key) => {
				self.0.suffix.push(Blake2_128Concat::hash(&key.encode()))
			},
			StorageItemKey::Twox64Concat(key) => {
				self.0.suffix.push(Twox64Concat::hash(&key.encode()))
			},
		}
	}

	pub fn build(mut self) -> StorageKey {
		let mut storage_key = Vec::new();
		// construct prefix
		let module_name = self.0.prefix.module;
		let storage_name = self.0.prefix.storage;
		storage_key.push(Twox128::hash(module_name.0.as_bytes()).to_vec());
		storage_key.push(Twox128::hash(storage_name.0.as_bytes()).to_vec());
		// add suffix
		storage_key.append(&mut self.0.suffix);

		StorageKey(storage_key.concat())
	}
}
