use super::config::Config;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ClientBuilder<'a, T> {
	fn from_config(config: &'a Config) -> Self;

	async fn build(self) -> T;
}

#[async_trait(?Send)]
pub trait Dispatcher<D, T> {
	async fn dispatch(&self, dispatchable: D) -> Result<T>;
}

#[async_trait(?Send)]
pub trait MutableDispatcher<D, T> {
	async fn dispatch(&mut self, dispatchable: D) -> Result<T>;
}
