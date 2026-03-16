pub mod anthropic;
pub mod openai;

use anyhow::Result;
use async_trait::async_trait;

use crate::models::{ChangeEntry, Provider};

#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    fn provider(&self) -> Provider;
    async fn fetch(&self) -> Result<Vec<ChangeEntry>>;
}

pub fn adapter_for(provider: &Provider) -> Box<dyn ProviderAdapter> {
    match provider {
        Provider::Anthropic => Box::new(anthropic::AnthropicAdapter),
        Provider::OpenAI => Box::new(openai::OpenAIAdapter),
        // TODO: implement remaining adapters
        _ => Box::new(anthropic::AnthropicAdapter), // placeholder
    }
}
