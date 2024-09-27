use std::{cell::RefCell, rc::Rc, sync::Arc};

pub type MutableRef<T> = Ref<RefCell<T>>;

pub fn create_mutable_ref<T>(value: T) -> MutableRef<T> {
	Rc::new(RefCell::new(value))
}

pub type Ref<T> = Rc<T>;

pub type AtomicRef<T> = Arc<T>;

pub fn create_atomic_ref<T>(value: T) -> AtomicRef<T> {
	Arc::new(value)
}
