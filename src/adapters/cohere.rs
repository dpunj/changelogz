use anyhow::Result;
use async_trait::async_trait;

use crate::models::{ChangeEntry, Provider};

use super::github;
use super::ProviderAdapter;

pub struct CohereAdapter;

#[async_trait]
impl ProviderAdapter for CohereAdapter {
    async fn fetch(&self) -> Result<Vec<ChangeEntry>> {
        let mut entries = Vec::new();

        // Cohere Python SDK
        match github::fetch_releases("cohere-ai", "cohere-python", Provider::Cohere, 30).await {
            Ok(mut e) => entries.append(&mut e),
            Err(err) => eprintln!("cohere python sdk: {}", err),
        }

        // Cohere TypeScript SDK
        match github::fetch_releases("cohere-ai", "cohere-typescript", Provider::Cohere, 30).await
        {
            Ok(mut e) => {
                for entry in &mut e {
                    entry.tags.push("typescript".to_string());
                }
                entries.append(&mut e);
            }
            Err(err) => eprintln!("cohere typescript sdk: {}", err),
        }

        entries.sort_by(|a, b| b.date.cmp(&a.date));
        Ok(entries)
    }
}
