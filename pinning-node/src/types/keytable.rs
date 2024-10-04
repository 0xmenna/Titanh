use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
    vec,
};

use super::cid::Cid;
use anyhow::Result;
use api::capsules_types::CapsuleKey;
use codec::{Decode, Encode};

/// Maximum number of columns that can be stored in the table. Assuming a (column, value) pair is approximately 76 bytes (key=>cid), a single row can handle approximately 256 MB.
pub const MAX_COLUMNS: u32 = 3368416;
/// Bounded BTreeMap that stores key-value pairs ordered by key. It is the row abstraction of the table.
#[derive(Encode, Decode, Clone)]
pub struct Row<C, V, const S: u32>(BTreeMap<C, V>);

impl<C, V, const S: u32> TryFrom<BTreeMap<C, V>> for Row<C, V, S> {
    type Error = ();

    fn try_from(map: BTreeMap<C, V>) -> Result<Self, Self::Error> {
        if map.len() as u32 <= S {
            Ok(Row(map))
        } else {
            Err(())
        }
    }
}

impl<C, V, const S: u32> Row<C, V, S> {
    pub fn new() -> Self {
        Row(BTreeMap::new())
    }
}

impl<K, V, const S: u32> Deref for Row<K, V, S> {
    type Target = BTreeMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V, const S: u32> DerefMut for Row<K, V, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub type OrderedRows<C, V, const S: u32> = Vec<Row<C, V, S>>;

#[derive(Encode, Decode, Clone)]
pub struct KeyTable<C, V, const S: u32>(OrderedRows<C, V, S>);

impl<C, V, const S: u32> KeyTable<C, V, S>
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

        let row_columns = row.len() as u32;

        if row_columns < Self::max_columns() || row.contains_key(&column_key) {
            let old_value = row.insert(column_key, value);
            Ok(old_value)
        } else {
            Err(anyhow::anyhow!(
                "Row has reached the maximum number of columns"
            ))
        }
    }

    pub fn add_row(&mut self, row: Row<C, V, S>) {
        for (idx, key_row) in self.0.iter_mut().enumerate() {
            if key_row.is_empty() {
                self.0[idx] = row;
                break;
            }
        }
    }

    pub fn insert_row_at(&mut self, row_idx: usize, row: Row<C, V, S>) -> Result<()> {
        if row_idx >= self.0.len() {
            return Err(anyhow::anyhow!("Row number out of bounds"));
        }

        self.0[row_idx] = row;

        Ok(())
    }

    fn put_front(&mut self, row: Row<C, V, S>) {
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

    fn mutable_row(&mut self, row_idx: usize) -> Result<&mut Row<C, V, S>> {
        let maybe_row = self.0.get_mut(row_idx);
        if let Some(row) = maybe_row {
            Ok(row)
        } else {
            Err(anyhow::anyhow!("Row number out of bounds"))
        }
    }

    pub fn row(&self, row_idx: usize) -> Result<&Row<C, V, S>> {
        let maybe_row = self.0.get(row_idx);
        if let Some(row) = maybe_row {
            Ok(row)
        } else {
            Err(anyhow::anyhow!("Row number out of bounds"))
        }
    }

    pub fn encoded_rows(&self) -> Vec<Vec<u8>> {
        self.0.iter().map(|row| row.encode()).collect()
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }

    pub fn max_columns() -> u32 {
        S
    }
}

type ColumnKey = CapsuleKey;

pub type TableRow = Row<ColumnKey, Cid, MAX_COLUMNS>;

#[derive(Encode, Decode, Clone)]
pub struct FaultTolerantKeyTable {
    /// The key table handled by the pinning node.
    /// Each row is a partition of the key space and is bounded to the replication factor.
    /// i.e. the first row is the closest key range to the node, the second row is the second closest key range, and so on, up to the replication factor.
    key_table: KeyTable<ColumnKey, Cid, MAX_COLUMNS>,
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
        let row = self.key_table.mutable_row(row_idx)?;
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

        let mut rows = Vec::new();
        while next_row_idx < self.rep_factor as usize {
            let mut row = self.key_table.row(curr_row_idx)?.clone();

            // take the next row and append it to the current row, leaving the next row empty
            let mut next_row = self.key_table.mutable_row(next_row_idx)?;
            let next_size = next_row.len() as u32;
            let left_size = MAX_COLUMNS - row.len() as u32;
            if left_size < next_size {
                return Err(anyhow::anyhow!("Not enough space to merge rows"));
            }

            row.append(&mut next_row);
            rows.push((curr_row_idx, row));

            curr_row_idx += 1;
            next_row_idx += 1;
        }

        // insert the merged rows back into the table
        for (idx, row) in rows {
            self.key_table.insert_row_at(idx, row)?;
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

    pub fn flush(&mut self) -> Vec<&TableRow> {
        let mut rows = Vec::new();
        for (idx, should_flush) in self.rows_to_flush.iter().enumerate() {
            if *should_flush {
                rows.push(self.key_table.row(idx).unwrap());
            }
        }

        self.rows_to_flush.fill(false);

        rows
    }

    pub fn extend_last_row(&mut self, row: &mut TableRow) -> Result<()> {
        let last_idx = self.key_table.size() - 1;
        let last_row = self.key_table.mutable_row(last_idx)?;

        let row_size = row.len() as u32;
        let left_size = MAX_COLUMNS - last_row.len() as u32;
        if left_size < row_size {
            return Err(anyhow::anyhow!("Not enough space to extend last row"));
        }

        last_row.append(row);

        self.rows_to_flush[last_idx] = true;

        Ok(())
    }

    pub fn mutable_table(&mut self) -> &mut KeyTable<ColumnKey, Cid, MAX_COLUMNS> {
        &mut self.key_table
    }

    pub fn encoded_rows(&self) -> Vec<Vec<u8>> {
        self.key_table.encoded_rows()
    }
}
