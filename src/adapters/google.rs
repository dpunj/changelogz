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

        // Google Generative AI Python SDK (Gemini)
        match github::fetch_releases(
            "google-gemini",
            "generative-ai-python",
            Provider::Google,
            30,
        )
        .await
        {
            Ok(mut e) => entries.append(&mut e),
            Err(err) => eprintln!("google genai python: {}", err),
        }

        // Google Generative AI JS SDK
        match github::fetch_releases("google-gemini", "generative-ai-js", Provider::Google, 30)
            .await
        {
            Ok(mut e) => {
                for entry in &mut e {
                    entry.tags.push("javascript".to_string());
                }
                entries.append(&mut e);
            }
            Err(err) => eprintln!("google genai js: {}", err),
        }

        entries.sort_by(|a, b| b.date.cmp(&a.date));
        Ok(entries)
    }
}
