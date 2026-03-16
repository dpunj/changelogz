use anyhow::Result;
use async_trait::async_trait;

use crate::models::{ChangeEntry, Provider};

use super::github;
use super::ProviderAdapter;

pub struct MistralAdapter;

#[async_trait]
impl ProviderAdapter for MistralAdapter {
    async fn fetch(&self) -> Result<Vec<ChangeEntry>> {
        let mut entries = Vec::new();

        // Mistral Python client
        match github::fetch_releases("mistralai", "client-python", Provider::Mistral, 30).await {
            Ok(mut e) => entries.append(&mut e),
            Err(err) => eprintln!("mistral python client: {}", err),
        }

        // Mistral JS client
        match github::fetch_releases("mistralai", "client-js", Provider::Mistral, 30).await {
            Ok(mut e) => {
                for entry in &mut e {
                    entry.tags.push("javascript".to_string());
                }
                entries.append(&mut e);
            }
            Err(err) => eprintln!("mistral js client: {}", err),
        }

        entries.sort_by(|a, b| b.date.cmp(&a.date));
        Ok(entries)
    }
}
