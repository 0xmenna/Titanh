use super::config::Config;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ClientBuilder<'a, T> {
	fn from_config(config: &'a Config) -> Self;

	async fn build(self) -> T;
}

#[async_trait(?Send)]
pub trait Dispatcher<E> {
	async fn dispatch(&self, event: &E) -> Result<()>;
}

#[async_trait(?Send)]
pub trait MutableDispatcher<E> {
	async fn dispatch(&mut self, event: &E) -> Result<()>;
}
