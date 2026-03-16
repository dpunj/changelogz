use crate::models::ChangeEntry;

pub enum OutputFormat {
    Human,
    Json,
}

pub fn print_entries(entries: &[ChangeEntry], format: &OutputFormat) {
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(entries).unwrap_or_default();
            println!("{}", json);
        }
        OutputFormat::Human => {
            if entries.is_empty() {
                println!("No entries found.");
                return;
            }
            for entry in entries {
                println!(
                    "\x1b[36m{}\x1b[0m  \x1b[33m{}\x1b[0m  \x1b[35m{:<12}\x1b[0m {}",
                    entry.date, entry.provider, format!("[{}]", entry.kind), entry.title
                );
            }
        }
    }
}

pub fn print_providers(providers: &[crate::models::Provider], subscribed: &[crate::models::Provider]) {
    println!("\x1b[1mProviders:\x1b[0m\n");
    for p in providers {
        let mark = if subscribed.contains(p) {
            "\x1b[32m●\x1b[0m"
        } else {
            "\x1b[90m○\x1b[0m"
        };
        println!("  {} {}", mark, p);
    }
    println!("\n  \x1b[32m●\x1b[0m = subscribed");
}
