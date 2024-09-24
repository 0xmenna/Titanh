use super::cid::Cid;
use anyhow::Result;
use api::{capsules_types::CapsuleKey, common_types::BlockNumber};
use codec::{Decode, Encode};
use sp_core::{bounded::BoundedBTreeMap, ConstU32};

/// Maximum number of columns that can be stored in the table. Assuming a (column, value) pair is approximately 70/80 bytes, a single row can handle 32 GB of data.
type MaxColumns = ConstU32<452_102_030>;
/// Bounded BTreeMap that stores key-value pairs ordered by key. It is the row abstraction of the table.
type Row<C, V> = BoundedBTreeMap<C, V, MaxColumns>;
type OrderedRows<C, V> = Vec<Row<C, V>>;

#[derive(Encode, Decode, Default, Clone)]
struct KeyTable<C, V>(OrderedRows<C, V>);

impl<C, V> KeyTable<C, V>
where
	C: Ord + Encode + Decode,
	V: Encode + Decode,
{
	pub fn new(rep_factor: u32) -> Self {
		let mut rows = OrderedRows::with_capacity(rep_factor as usize);
		for _ in 0..rep_factor {
			rows.push(Row::new());
		}

		KeyTable(rows)
	}

	pub fn insert(&mut self, row_num: usize, column_key: C, value: V) -> Result<Option<V>> {
		let row = self.get_mutable_row(row_num)?;
		let old_value = row.insert(column_key, value);

		Ok(old_value)
	}

	pub fn remove(&mut self, row_num: usize, column_key: &C) -> Result<Option<V>> {
		let row = self.get_mutable_row(row_num)?;
		let rm_value = row.remove(column_key);

		Ok(rm_value)
	}

	pub fn get(&self, row_num: usize, column_key: &C) -> Result<Option<&V>> {
		let row = self.get_row(row_num)?;
		let value = row.get(column_key);

		Ok(value)
	}

	fn get_mutable_row(&mut self, row_num: usize) -> Result<&mut Row<C, V>> {
		let maybe_row = self.0.get_mut(row_num);
		if let Some(row) = maybe_row {
			Ok(row)
		} else {
			Err(anyhow::anyhow!("Row number out of bounds"))
		}
	}

	pub fn get_row(&mut self, row_num: usize) -> Result<&Row<C, V>> {
		let maybe_row = self.0.get(row_num);
		if let Some(row) = maybe_row {
			Ok(row)
		} else {
			Err(anyhow::anyhow!("Row number out of bounds"))
		}
	}
}

#[derive(Encode, Decode, Clone)]
pub struct FaultTolerantKeyTable {
	at: BlockNumber,
	/// The key table handled by the pinning node.
	/// Each row is a partition of the key space and is bounded to the replication factor.
	/// i.e. the first row is the closest key range to the node, the second row is the second closest key range, and so on, up to the replication factor.
	key_table: KeyTable<CapsuleKey, Cid>,
	rep_factor: u32,
}

impl FaultTolerantKeyTable {
	pub fn new(rep_factor: u32) -> Self {
		FaultTolerantKeyTable { at: 0, key_table: KeyTable::new(rep_factor), rep_factor }
	}

	pub fn snapshot(&mut self, snapshot: BlockNumber) {
		self.at = snapshot;
	}

	pub fn at(&self) -> BlockNumber {
		self.at
	}
}
