use crate::models::{ChangeEntry, ChangeKind};

pub enum OutputFormat {
    Human,
    Json,
}

fn kind_color(kind: &ChangeKind) -> &str {
    match kind {
        ChangeKind::Breaking => "\x1b[38;2;220;80;80m",
        ChangeKind::Deprecation => "\x1b[38;2;220;180;50m",
        ChangeKind::Feature => "\x1b[38;2;80;200;120m",
        ChangeKind::ModelRelease => "\x1b[38;2;180;120;220m",
        ChangeKind::Fix => "\x1b[38;2;100;150;240m",
        ChangeKind::Other => "\x1b[38;2;120;120;120m",
    }
}

const CYAN: &str = "\x1b[38;2;86;182;194m";
const DIM: &str = "\x1b[38;2;120;120;120m";
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";

pub fn print_entries(entries: &[ChangeEntry], format: &OutputFormat) {
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(entries).unwrap_or_default();
            println!("{}", json);
        }
        OutputFormat::Human => {
            if entries.is_empty() {
                println!("{}No entries found.{}", DIM, RESET);
                println!("{}Subscribe and fetch first: changelogz sub <provider> && changelogz fetch{}", DIM, RESET);
                return;
            }
            for entry in entries {
                let kc = kind_color(&entry.kind);
                let kind_label: String = format!("{}", entry.kind).chars().take(5).collect();
                println!(
                    "{}{}{} {}{:<4}{} {}{:<6}{} {}",
                    DIM, entry.date, RESET,
                    CYAN, &entry.provider.id()[..3].to_uppercase(), RESET,
                    kc, kind_label, RESET,
                    entry.title,
                );
            }
        }
    }
}

pub fn print_providers(providers: &[crate::models::Provider], subscribed: &[crate::models::Provider]) {
    println!("\n {}{}Providers{}\n", BOLD, CYAN, RESET);
    for p in providers {
        let (mark, color) = if subscribed.contains(p) {
            ("●", "\x1b[38;2;80;200;120m")
        } else {
            ("○", DIM)
        };
        println!("   {}{}{} {}", color, mark, RESET, p);
    }
    println!();
}
