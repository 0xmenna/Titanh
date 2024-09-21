use std::{cell::RefCell, rc::Rc, sync::Arc};

use tokio::sync::Mutex;

pub type MutableRef<T> = Ref<RefCell<T>>;

pub fn create_mutable_ref<T>(value: T) -> MutableRef<T> {
	Rc::new(RefCell::new(value))
}

pub type Ref<T> = Rc<T>;

pub fn create_ref<T>(value: T) -> Ref<T> {
	Rc::new(value)
}

pub type AtomicRef<T> = Arc<T>;

pub fn create_atomic_ref<T>(value: T) -> AtomicRef<T> {
	Arc::new(value)
}

pub type MutableAtomicRef<T> = Arc<Mutex<T>>;

pub fn create_mutable_atomic_ref<T>(value: T) -> MutableAtomicRef<T> {
	Arc::new(Mutex::new(value))
}
