use super::config::Config;
use async_trait::async_trait;

#[async_trait]
pub trait ClientBuilder<'a, T> {
	fn from_config(config: &'a Config) -> Self;

	async fn build(self) -> T;
}
