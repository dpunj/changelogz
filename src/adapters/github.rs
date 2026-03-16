use anyhow::Result;
use chrono::NaiveDate;

use crate::models::{ChangeEntry, Provider};

#[derive(serde::Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub html_url: String,
    pub published_at: Option<String>,
}

/// Fetch releases from a GitHub repo and map to ChangeEntries
pub async fn fetch_releases(
    owner: &str,
    repo: &str,
    provider: Provider,
    per_page: u8,
) -> Result<Vec<ChangeEntry>> {
    let client = reqwest::Client::builder()
        .user_agent("changelogz/0.1")
        .build()?;

    let url = format!(
        "https://api.github.com/repos/{}/{}/releases?per_page={}",
        owner, repo, per_page
    );

    let resp = client.get(&url).send().await?;

    if !resp.status().is_success() {
        anyhow::bail!(
            "GitHub API returned {} for {}/{}",
            resp.status(),
            owner,
            repo
        );
    }

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
                provider: provider.clone(),
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
