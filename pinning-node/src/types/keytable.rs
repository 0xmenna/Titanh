use std::vec;

use super::{cid::Cid, events::PinningEvent};
use crate::utils::traits::MutableDispatcher;
use anyhow::Result;
use api::capsules_types::CapsuleKey;
use async_trait::async_trait;
use codec::{Decode, Encode};
use sp_core::{bounded::BoundedBTreeMap, crypto::UncheckedInto, ConstU32, H256};

/// Maximum number of columns that can be stored in the table. Assuming a (column, value) pair is approximately 70/80 bytes, a single row can handle 32 GB of data.
type MaxColumns = ConstU32<452_102_030>;
/// Bounded BTreeMap that stores key-value pairs ordered by key. It is the row abstraction of the table.
pub type Row<C, V> = BoundedBTreeMap<C, V, MaxColumns>;
pub type OrderedRows<C, V> = Vec<Row<C, V>>;

#[derive(Encode, Decode, Clone)]
pub struct KeyTable<C, V>(OrderedRows<C, V>);

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

	pub fn insert(&mut self, row_idx: usize, column_key: C, value: V) -> Result<Option<V>> {
		let row = self.mutable_row(row_idx)?;
		let old_value = row.insert(column_key, value);

		Ok(old_value)
	}

	pub fn add_row(&mut self, row: Row<C, V>) {
		for (idx, key_row) in self.0.iter().enumerate() {
			if key_row.is_empty() {
				self.0[idx] = row;
			}
		}
	}

	fn put_front(&mut self, row: Row<C, V>) {
		let rows_num = self.0.len();
		if rows_num == self.0.capacity() {
			self.0.remove(rows_num - 1);
		}
		self.0.insert(0, row);
	}

	pub fn remove(&mut self, row_idx: usize, column_key: &C) -> Result<Option<V>> {
		let row = self.mutable_row(row_idx)?;
		let rm_value = row.remove(column_key);

		Ok(rm_value)
	}

	pub fn get(&self, row_idx: usize, column_key: &C) -> Result<Option<&V>> {
		let row = self.row(row_idx)?;
		let value = row.get(column_key);

		Ok(value)
	}

	fn mutable_row(&mut self, row_idx: usize) -> Result<&mut Row<C, V>> {
		let maybe_row = self.0.get_mut(row_idx);
		if let Some(row) = maybe_row {
			Ok(row)
		} else {
			Err(anyhow::anyhow!("Row number out of bounds"))
		}
	}

	pub fn row(&self, row_idx: usize) -> Result<&Row<C, V>> {
		let maybe_row = self.0.get(row_idx);
		if let Some(row) = maybe_row {
			Ok(row)
		} else {
			Err(anyhow::anyhow!("Row number out of bounds"))
		}
	}

	pub fn size(&self) -> usize {
		self.0.len()
	}
}

type ColumnKey = CapsuleKey;

#[derive(Encode, Decode, Clone)]
pub struct FaultTolerantKeyTable {
	/// The key table handled by the pinning node.
	/// Each row is a partition of the key space and is bounded to the replication factor.
	/// i.e. the first row is the closest key range to the node, the second row is the second closest key range, and so on, up to the replication factor.
	key_table: KeyTable<ColumnKey, Cid>,
	rep_factor: u32,
	rows_to_flush: Vec<bool>,
}

impl FaultTolerantKeyTable {
	pub fn new(rep_factor: u32) -> Self {
		FaultTolerantKeyTable {
			key_table: KeyTable::new(rep_factor),
			rep_factor,
			rows_to_flush: vec![false; rep_factor as usize],
		}
	}

	pub fn partition_row(&mut self, row_idx: usize, barrier_key: &ColumnKey) -> Result<()> {
		let mut row = self.key_table.mutable_row(row_idx)?;
		let mut new_row = row.split_off(barrier_key);
		// since `split_off` includes the barrier key in the new row, we need to swap it with the original row
		let maybe_val = new_row.remove(barrier_key);
		if let Some(val) = maybe_val {
			row.insert(*barrier_key, val);
		}

		// must be bounded
		let new_row = new_row.try_into().unwrap();

		// We need to shift existing rows to the right to make space for the new row and remove the last row (if any). Insertion at front is O(n), but since we assume the number of rows is not large, it is acceptable.
		self.key_table.put_front(new_row);

		self.rows_to_flush.fill(true);

		Ok(())
	}

	pub fn merge_rows_from(&mut self, row_idx: usize) -> Result<()> {
		let mut curr_row_idx = row_idx;
		let mut next_row_idx = row_idx + 1;

		while next_row_idx < self.rep_factor as usize {
			let mut row = self.key_table.mutable_row(curr_row_idx)?;

			// take the next row and append it to the current row, leaving the next row empty
			let mut next_row = self.key_table.mutable_row(next_row_idx)?;
			row.append(&mut next_row);

			curr_row_idx += 1;
			next_row_idx += 1;
		}

		self.rows_to_flush.fill(true);
		// at this point the last row is surely empty, leaving room for a new row transferred from another node

		Ok(())
	}

	pub fn insert(
		&mut self,
		row_idx: usize,
		column_key: ColumnKey,
		value: Cid,
	) -> Result<Option<Cid>> {
		let val = self.key_table.insert(row_idx, column_key, value)?;
		self.rows_to_flush[row_idx] = true;

		Ok(val)
	}

	pub fn remove(&mut self, row_idx: usize, column_key: &ColumnKey) -> Result<Option<Cid>> {
		let val = self.key_table.remove(row_idx, column_key)?;
		self.rows_to_flush[row_idx] = true;

		Ok(val)
	}

	pub fn flush(&mut self) -> Vec<&Row<ColumnKey, Cid>> {
		let mut rows = Vec::new();
		for (idx, should_flush) in self.rows_to_flush.iter().enumerate() {
			if *should_flush {
				rows.push(self.key_table.row(idx).unwrap());
			}
		}

		self.rows_to_flush.fill(false);

		rows
	}

	pub fn add_row(&mut self, row: Row<ColumnKey, Cid>) {
		self.key_table.add_row(row);
	}

	pub fn extend_last_row(&mut self, row: &mut Row<ColumnKey, Cid>) -> Result<()> {
		let last_idx = self.key_table.size() - 1;
		let mut last_row = self.key_table.mutable_row(last_idx)?;
		last_row.append(row);

		self.rows_to_flush[last_idx] = true;

		Ok(())
	}
}
