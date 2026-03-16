use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::PathBuf;

use crate::models::{ChangeEntry, ChangeKind, Provider};

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open() -> Result<Self> {
        let path = Self::db_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)?;
        let store = Store { conn };
        store.migrate()?;
        Ok(store)
    }

    fn db_path() -> Result<PathBuf> {
        let dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("changeloz");
        Ok(dir.join("changeloz.db"))
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS subscriptions (
                provider TEXT PRIMARY KEY
            );
            CREATE TABLE IF NOT EXISTS entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                provider TEXT NOT NULL,
                date TEXT NOT NULL,
                title TEXT NOT NULL,
                body TEXT NOT NULL,
                kind TEXT NOT NULL,
                url TEXT NOT NULL,
                tags TEXT NOT NULL,
                UNIQUE(provider, url)
            );
            CREATE INDEX IF NOT EXISTS idx_entries_date ON entries(date DESC);
            CREATE INDEX IF NOT EXISTS idx_entries_provider ON entries(provider);
            ",
        )?;
        Ok(())
    }

    // --- Subscriptions ---

    pub fn subscribe(&self, provider: &Provider) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO subscriptions (provider) VALUES (?1)",
            params![provider.id()],
        )?;
        Ok(())
    }

    pub fn unsubscribe(&self, provider: &Provider) -> Result<()> {
        self.conn.execute(
            "DELETE FROM subscriptions WHERE provider = ?1",
            params![provider.id()],
        )?;
        Ok(())
    }

    pub fn subscriptions(&self) -> Result<Vec<Provider>> {
        let mut stmt = self.conn.prepare("SELECT provider FROM subscriptions")?;
        let providers = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .filter_map(|s| Provider::from_str(&s))
            .collect();
        Ok(providers)
    }

    #[allow(dead_code)]
    pub fn is_subscribed(&self, provider: &Provider) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM subscriptions WHERE provider = ?1",
            params![provider.id()],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    // --- Entries ---

    pub fn upsert_entries(&self, entries: &[ChangeEntry]) -> Result<usize> {
        let mut count = 0;
        for entry in entries {
            let tags_json = serde_json::to_string(&entry.tags)?;
            let inserted = self.conn.execute(
                "INSERT OR IGNORE INTO entries (provider, date, title, body, kind, url, tags)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    entry.provider.id(),
                    entry.date.to_string(),
                    entry.title,
                    entry.body,
                    format!("{}", entry.kind).to_lowercase(),
                    entry.url,
                    tags_json,
                ],
            )?;
            count += inserted;
        }
        Ok(count)
    }

    pub fn get_feed(
        &self,
        providers: Option<&[Provider]>,
        kind: Option<&ChangeKind>,
        limit: usize,
    ) -> Result<Vec<ChangeEntry>> {
        let mut sql = String::from(
            "SELECT provider, date, title, body, kind, url, tags FROM entries WHERE 1=1",
        );
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(provs) = providers {
            if !provs.is_empty() {
                let placeholders: Vec<String> = provs.iter().map(|p| {
                    bind_values.push(p.id().to_string());
                    format!("?{}", bind_values.len())
                }).collect();
                sql.push_str(&format!(" AND provider IN ({})", placeholders.join(",")));
            }
        }

        if let Some(k) = kind {
            bind_values.push(format!("{}", k).to_lowercase());
            sql.push_str(&format!(" AND kind = ?{}", bind_values.len()));
        }

        sql.push_str(&format!(" ORDER BY date DESC LIMIT {}", limit));

        let mut stmt = self.conn.prepare(&sql)?;

        let entries = stmt
            .query_map(
                rusqlite::params_from_iter(bind_values.iter()),
                |row| {
                    let provider_str: String = row.get(0)?;
                    let date_str: String = row.get(1)?;
                    let title: String = row.get(2)?;
                    let body: String = row.get(3)?;
                    let kind_str: String = row.get(4)?;
                    let url: String = row.get(5)?;
                    let tags_str: String = row.get(6)?;

                    Ok((provider_str, date_str, title, body, kind_str, url, tags_str))
                },
            )?
            .filter_map(|r| r.ok())
            .filter_map(|(provider_str, date_str, title, body, kind_str, url, tags_str)| {
                let provider = Provider::from_str(&provider_str)?;
                let date = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok()?;
                let kind = match kind_str.as_str() {
                    "breaking" => ChangeKind::Breaking,
                    "deprecation" => ChangeKind::Deprecation,
                    "feature" => ChangeKind::Feature,
                    "model" => ChangeKind::ModelRelease,
                    "fix" => ChangeKind::Fix,
                    _ => ChangeKind::Other,
                };
                let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();

                Some(ChangeEntry {
                    provider,
                    date,
                    title,
                    body,
                    kind,
                    url,
                    tags,
                })
            })
            .collect();

        Ok(entries)
    }
}
