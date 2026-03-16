pub mod anthropic;
pub mod cohere;
pub mod github;
pub mod google;
pub mod mistral;
pub mod openai;

use anyhow::Result;
use async_trait::async_trait;

use crate::models::{ChangeEntry, Provider};

#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    async fn fetch(&self) -> Result<Vec<ChangeEntry>>;
}

pub fn adapter_for(provider: &Provider) -> Box<dyn ProviderAdapter> {
    match provider {
        Provider::Anthropic => Box::new(anthropic::AnthropicAdapter),
        Provider::OpenAI => Box::new(openai::OpenAIAdapter),
        Provider::Google => Box::new(google::GoogleAdapter),
        Provider::Mistral => Box::new(mistral::MistralAdapter),
        Provider::Cohere => Box::new(cohere::CohereAdapter),
    }
}
