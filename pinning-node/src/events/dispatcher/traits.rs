use anyhow::Result;
use async_trait::async_trait;

pub trait Dispatcher<D, T> {
	fn dispatch(&self, dispatchable: D) -> Result<T>;
}

pub trait MutableDispatcher<D, T> {
	fn dispatch(&mut self, dispatchable: D) -> Result<T>;
}

#[async_trait(?Send)]
pub trait AsyncDispatcher<D, T> {
	async fn async_dispatch(&self, dispatchable: D) -> Result<T>;
}

#[async_trait(?Send)]
pub trait AsyncMutableDispatcher<D, T> {
	async fn async_dispatch(&mut self, dispatchable: D) -> Result<T>;
}
