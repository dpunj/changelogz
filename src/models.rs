use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Anthropic,
    OpenAI,
    Google,
    Mistral,
    Cohere,
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Provider::Anthropic => write!(f, "Anthropic"),
            Provider::OpenAI => write!(f, "OpenAI"),
            Provider::Google => write!(f, "Google"),
            Provider::Mistral => write!(f, "Mistral"),
            Provider::Cohere => write!(f, "Cohere"),
        }
    }
}

impl Provider {
    pub fn all() -> Vec<Provider> {
        vec![
            Provider::Anthropic,
            Provider::OpenAI,
            Provider::Google,
            Provider::Mistral,
            Provider::Cohere,
        ]
    }

    pub fn from_str(s: &str) -> Option<Provider> {
        match s.to_lowercase().as_str() {
            "anthropic" => Some(Provider::Anthropic),
            "openai" => Some(Provider::OpenAI),
            "google" | "gemini" => Some(Provider::Google),
            "mistral" => Some(Provider::Mistral),
            "cohere" => Some(Provider::Cohere),
            _ => None,
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Provider::Anthropic => "anthropic",
            Provider::OpenAI => "openai",
            Provider::Google => "google",
            Provider::Mistral => "mistral",
            Provider::Cohere => "cohere",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChangeKind {
    Breaking,
    Deprecation,
    Feature,
    ModelRelease,
    Fix,
    Other,
}

impl fmt::Display for ChangeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChangeKind::Breaking => write!(f, "BREAKING"),
            ChangeKind::Deprecation => write!(f, "DEPRECATION"),
            ChangeKind::Feature => write!(f, "FEATURE"),
            ChangeKind::ModelRelease => write!(f, "MODEL"),
            ChangeKind::Fix => write!(f, "FIX"),
            ChangeKind::Other => write!(f, "OTHER"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEntry {
    pub provider: Provider,
    pub date: NaiveDate,
    pub title: String,
    pub body: String,
    pub kind: ChangeKind,
    pub url: String,
    pub tags: Vec<String>,
}

impl ChangeEntry {
    /// Classify a change entry based on title/body keywords
    pub fn classify(title: &str, body: &str) -> ChangeKind {
        let text = format!("{} {}", title, body).to_lowercase();

        if text.contains("breaking") || text.contains("removed") || text.contains("incompatible") {
            ChangeKind::Breaking
        } else if text.contains("deprecat") {
            ChangeKind::Deprecation
        } else if text.contains("model") && (text.contains("release") || text.contains("launch") || text.contains("available")) {
            ChangeKind::ModelRelease
        } else if text.contains("fix") || text.contains("bug") || text.contains("patch") {
            ChangeKind::Fix
        } else if text.contains("new") || text.contains("feature") || text.contains("add") || text.contains("support") {
            ChangeKind::Feature
        } else {
            ChangeKind::Other
        }
    }
}
