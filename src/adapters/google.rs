use anyhow::Result;
use async_trait::async_trait;

use crate::models::{ChangeEntry, Provider};

use super::github;
use super::ProviderAdapter;

pub struct GoogleAdapter;

#[async_trait]
impl ProviderAdapter for GoogleAdapter {
    async fn fetch(&self) -> Result<Vec<ChangeEntry>> {
        let mut entries = Vec::new();

        // Google Gen AI Python SDK (new, active)
        match github::fetch_releases("googleapis", "python-genai", Provider::Google, 30).await {
            Ok(mut e) => entries.append(&mut e),
            Err(err) => eprintln!("google python-genai: {}", err),
        }

        // Google Gen AI JS/TS SDK (new, active)
        match github::fetch_releases("googleapis", "js-genai", Provider::Google, 30).await {
            Ok(mut e) => {
                for entry in &mut e {
                    entry.tags.push("javascript".to_string());
                }
                entries.append(&mut e);
            }
            Err(err) => eprintln!("google js-genai: {}", err),
        }

        entries.sort_by(|a, b| b.date.cmp(&a.date));
        Ok(entries)
    }
}
