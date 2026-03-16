mod adapters;
mod models;
mod output;
mod store;
mod tui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use models::{ChangeKind, Provider};
use output::OutputFormat;
use store::Store;

#[derive(Parser)]
#[command(name = "changeloz", about = "Track LLM API changelog updates", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Subscribe to a provider
    Sub {
        /// Provider name (anthropic, openai, google, mistral, cohere)
        provider: String,
    },
    /// Unsubscribe from a provider
    Unsub {
        /// Provider name
        provider: String,
    },
    /// List providers and subscription status
    List,
    /// Fetch and display the changelog feed
    Feed {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Filter by provider
        #[arg(short, long)]
        provider: Option<String>,

        /// Filter by change kind (breaking, deprecation, feature, model, fix)
        #[arg(short, long)]
        kind: Option<String>,

        /// Max entries to show
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },
    /// Fetch latest changes from subscribed providers
    Fetch,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => {
            // Default: launch TUI
            tui::run_tui()?;
        }
        Some(Commands::Sub { provider }) => {
            let p = Provider::from_str(&provider)
                .ok_or_else(|| anyhow::anyhow!("Unknown provider: {}", provider))?;
            let store = Store::open()?;
            store.subscribe(&p)?;
            println!("✓ Subscribed to {}", p);
        }
        Some(Commands::Unsub { provider }) => {
            let p = Provider::from_str(&provider)
                .ok_or_else(|| anyhow::anyhow!("Unknown provider: {}", provider))?;
            let store = Store::open()?;
            store.unsubscribe(&p)?;
            println!("✓ Unsubscribed from {}", p);
        }
        Some(Commands::List) => {
            let store = Store::open()?;
            let subscribed = store.subscriptions()?;
            output::print_providers(&Provider::all(), &subscribed);
        }
        Some(Commands::Feed { json, provider, kind, limit }) => {
            let store = Store::open()?;

            let providers = match provider {
                Some(ref name) => {
                    let p = Provider::from_str(name)
                        .ok_or_else(|| anyhow::anyhow!("Unknown provider: {}", name))?;
                    Some(vec![p])
                }
                None => None,
            };

            let change_kind = kind.as_deref().and_then(|k| match k {
                "breaking" => Some(ChangeKind::Breaking),
                "deprecation" => Some(ChangeKind::Deprecation),
                "feature" => Some(ChangeKind::Feature),
                "model" => Some(ChangeKind::ModelRelease),
                "fix" => Some(ChangeKind::Fix),
                _ => None,
            });

            let format = if json { OutputFormat::Json } else { OutputFormat::Human };
            let entries = store.get_feed(
                providers.as_deref(),
                change_kind.as_ref(),
                limit,
            )?;
            output::print_entries(&entries, &format);
        }
        Some(Commands::Fetch) => {
            let store = Store::open()?;
            let subscribed = store.subscriptions()?;

            if subscribed.is_empty() {
                println!("No subscriptions. Use `changeloz sub <provider>` first.");
                return Ok(());
            }

            for provider in &subscribed {
                print!("Fetching {}... ", provider);
                let adapter = adapters::adapter_for(provider);
                match adapter.fetch().await {
                    Ok(entries) => {
                        let count = store.upsert_entries(&entries)?;
                        println!("✓ {} new entries ({} total)", count, entries.len());
                    }
                    Err(e) => {
                        println!("✗ Error: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}
