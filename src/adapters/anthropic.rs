use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDate;
use scraper::{Html, Selector};

use crate::models::{ChangeEntry, Provider};

use super::ProviderAdapter;

pub struct AnthropicAdapter;

const API_CHANGELOG_URL: &str = "https://docs.anthropic.com/en/api/changelog";

#[async_trait]
impl ProviderAdapter for AnthropicAdapter {
    fn provider(&self) -> Provider {
        Provider::Anthropic
    }

    async fn fetch(&self) -> Result<Vec<ChangeEntry>> {
        // Try the API changelog page first
        let entries = fetch_api_changelog().await?;
        if !entries.is_empty() {
            return Ok(entries);
        }

        // Fallback: fetch from GitHub releases (Anthropic SDK)
        fetch_github_releases().await
    }
}

async fn fetch_api_changelog() -> Result<Vec<ChangeEntry>> {
    let client = reqwest::Client::builder()
        .user_agent("changeloz/0.1")
        .build()?;

    let resp = client.get(API_CHANGELOG_URL).send().await?;
    let html = resp.text().await?;
    let document = Html::parse_document(&html);

    let mut entries = Vec::new();

    // Look for date-headed sections — common pattern in changelog pages
    // Try h2 or h3 elements that contain dates
    let heading_sel = Selector::parse("h2, h3").unwrap();

    for heading in document.select(&heading_sel) {
        let heading_text = heading.text().collect::<String>().trim().to_string();

        // Try to parse as date (various formats)
        let date = parse_date_fuzzy(&heading_text);
        if date.is_none() {
            continue;
        }
        let date = date.unwrap();

        // Collect text from siblings until next heading
        let mut body_parts = Vec::new();
        let mut sibling = heading.next_sibling();
        while let Some(node) = sibling {
            if let Some(element) = node.value().as_element() {
                let tag = element.name();
                if tag == "h2" || tag == "h3" {
                    break;
                }
            }
            if let Some(text) = node.value().as_text() {
                let t = text.trim();
                if !t.is_empty() {
                    body_parts.push(t.to_string());
                }
            }
            // Also grab inner text from element children
            if node.value().is_element() {
                let text: String = node.descendants()
                    .filter_map(|n| n.value().as_text().map(|t| t.to_string()))
                    .collect::<Vec<_>>()
                    .join(" ");
                let t = text.trim().to_string();
                if !t.is_empty() {
                    body_parts.push(t);
                }
            }
            sibling = node.next_sibling();
        }

        let body = body_parts.join("\n");
        if body.is_empty() {
            continue;
        }

        let title = first_line_or_truncate(&body, 120);
        let kind = ChangeEntry::classify(&title, &body);

        entries.push(ChangeEntry {
            provider: Provider::Anthropic,
            date,
            title,
            body,
            kind,
            url: format!("{}#{}", API_CHANGELOG_URL, heading_text.to_lowercase().replace(' ', "-")),
            tags: vec!["api".to_string()],
        });
    }

    Ok(entries)
}

async fn fetch_github_releases() -> Result<Vec<ChangeEntry>> {
    let client = reqwest::Client::builder()
        .user_agent("changeloz/0.1")
        .build()?;

    // Fetch from Anthropic Python SDK releases as a proxy for API changes
    let url = "https://api.github.com/repos/anthropics/anthropic-sdk-python/releases?per_page=30";
    let resp = client.get(url).send().await?;
    let releases: Vec<GithubRelease> = resp.json().await?;

    let entries = releases
        .into_iter()
        .filter_map(|release| {
            let date = NaiveDate::parse_from_str(
                release.published_at.as_deref().unwrap_or("").get(..10)?,
                "%Y-%m-%d",
            )
            .ok()?;

            let title = release.name.unwrap_or_else(|| release.tag_name.clone());
            let body = release.body.unwrap_or_default();
            let kind = ChangeEntry::classify(&title, &body);

            Some(ChangeEntry {
                provider: Provider::Anthropic,
                date,
                title,
                body,
                kind,
                url: release.html_url,
                tags: vec!["sdk".to_string(), "github".to_string()],
            })
        })
        .collect();

    Ok(entries)
}

#[derive(serde::Deserialize)]
struct GithubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    html_url: String,
    published_at: Option<String>,
}

fn parse_date_fuzzy(s: &str) -> Option<NaiveDate> {
    // Try common formats
    let formats = [
        "%Y-%m-%d",
        "%B %d, %Y",   // January 15, 2025
        "%b %d, %Y",   // Jan 15, 2025
        "%d %B %Y",    // 15 January 2025
        "%m/%d/%Y",
    ];

    let cleaned = s.trim();
    for fmt in &formats {
        if let Ok(d) = NaiveDate::parse_from_str(cleaned, fmt) {
            return Some(d);
        }
    }
    None
}

fn first_line_or_truncate(s: &str, max: usize) -> String {
    let first_line = s.lines().next().unwrap_or(s);
    if first_line.len() <= max {
        first_line.to_string()
    } else {
        format!("{}...", &first_line[..max - 3])
    }
}
