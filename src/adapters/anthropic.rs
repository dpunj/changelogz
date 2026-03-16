use anyhow::Result;
use async_trait::async_trait;

use crate::models::{ChangeEntry, Provider};

use super::github;
use super::ProviderAdapter;

pub struct AnthropicAdapter;

#[async_trait]
impl ProviderAdapter for AnthropicAdapter {
    async fn fetch(&self) -> Result<Vec<ChangeEntry>> {
        let mut entries = Vec::new();

        // Python SDK releases
        match github::fetch_releases("anthropics", "anthropic-sdk-python", Provider::Anthropic, 30)
            .await
        {
            Ok(mut e) => entries.append(&mut e),
            Err(err) => eprintln!("anthropic python sdk: {}", err),
        }

        // TypeScript SDK releases
        match github::fetch_releases(
            "anthropics",
            "anthropic-sdk-typescript",
            Provider::Anthropic,
            30,
        )
        .await
        {
            Ok(mut e) => {
                // Tag these so we can distinguish
                for entry in &mut e {
                    entry.tags.push("typescript".to_string());
                }
                entries.append(&mut e);
            }
            Err(err) => eprintln!("anthropic ts sdk: {}", err),
        }

        // Sort by date descending
        entries.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(entries)
    }
}
