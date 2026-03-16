# changelogz

A lazygit-style TUI + CLI for tracking LLM API changelog updates.

Subscribe to providers, fetch changes, and browse a unified feed — from your terminal.

## Install

```bash
cargo install --path .
```

## Usage

```bash
# Launch the TUI
changelogz

# Subscribe to providers
changelogz sub anthropic
changelogz sub openai

# Fetch latest changes
changelogz fetch

# Browse the feed
changelogz feed
changelogz feed --kind breaking
changelogz feed --provider anthropic --json
changelogz feed --limit 20

# List providers
changelogz list

# Unsubscribe
changelogz unsub openai
```

## TUI Keybindings

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Switch panel (Providers ↔ Feed ↔ Detail) |
| `j/k` or `↑/↓` | Navigate |
| `g` / `G` | Jump to top / bottom |
| `Enter` | Toggle subscription (in Providers panel) |
| `/` | Search (fuzzy filter across title, body, tags) |
| `Esc` | Clear search |
| `o` | Open selected entry URL in browser |
| `r` | Fetch latest from subscribed providers |
| `1-5` | Filter by kind (Breaking/Deprecation/Feature/Model/Fix) |
| `0` | Clear kind filter |
| `q` | Quit |

## Providers (V1)

- Anthropic (via GitHub SDK releases)
- OpenAI (via GitHub SDK releases)
- Google (coming soon)
- Mistral (coming soon)
- Cohere (coming soon)

## Architecture

```
src/
  main.rs          — CLI entrypoint (clap)
  tui/             — ratatui TUI (lazygit-style panels)
  adapters/        — per-provider changelog fetchers
  models.rs        — ChangeEntry, Provider, ChangeKind
  store.rs         — local SQLite for subscriptions + cache
  output.rs        — JSON/human output formatters
```
