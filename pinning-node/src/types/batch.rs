#[derive(Clone)]
pub struct Batch<T>(Vec<T>);

impl<T> Default for Batch<T> {
	fn default() -> Self {
		Self(Vec::new())
	}
}

impl<T> From<Vec<T>> for Batch<T> {
	fn from(vec: Vec<T>) -> Self {
		Self(vec)
	}
}

impl<T> IntoIterator for Batch<T> {
	type Item = T;
	type IntoIter = std::vec::IntoIter<T>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<T> Batch<T> {
	pub fn insert(&mut self, item: T) {
		self.0.push(item);
	}

	pub fn extend(&mut self, batch: Self) {
		self.0.extend(batch.0);
	}

	pub fn clear(&mut self) {
		self.0.clear();
	}

	pub fn iter(&self) -> std::slice::Iter<'_, T> {
		self.0.iter()
	}
}
