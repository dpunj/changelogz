# changeloz — API Changelog TUI

## Frame

**Source**: Divesh's idea — no good CLI-native tool for tracking LLM API changes
**Problem**: Developers and agents have no unified, structured way to track API changelog updates from LLM providers. You either check docs manually, set up RSS/email, or miss breaking changes.
**Outcome**: A lazygit-quality Rust TUI + agent-friendly CLI that lets you subscribe to LLM provider changelogs and get a unified, structured feed.

## Requirements

- R1: Subscribe/unsubscribe to LLM API providers
- R2: Unified changelog feed across all subscribed providers
- R3: Filter by change type (breaking, deprecation, new feature, model release)
- R4: Agent-friendly output (`--json`, MCP server potential)
- R5: lazygit-level TUI experience (keyboard-driven, fast, beautiful)
- R6: Single binary, no runtime dependencies

## Providers (V1 — LLM-native only)

| Provider | Potential data sources |
|----------|----------------------|
| Anthropic | GitHub releases, docs changelog, OpenAPI spec |
| OpenAI | GitHub releases, docs changelog, OpenAPI spec |
| Google (Gemini) | Docs changelog, release notes |
| Mistral | GitHub releases, docs changelog |
| Cohere | Docs changelog |

## Shape: Provider Adapter Pattern

Each provider gets an adapter that knows how to fetch + normalize changelogs into a common `ChangeEntry` format:

```
ChangeEntry {
  provider: String,
  date: Date,
  title: String,
  body: String,
  kind: Breaking | Deprecation | Feature | ModelRelease | Fix,
  url: String,
  tags: Vec<String>,
}
```

Adapters pull from the best available source per provider (GitHub releases atom feed, RSS, OpenAPI spec diffs).

## Architecture (high-level)

```
changeloz/
  src/
    main.rs          — CLI entrypoint (clap)
    tui/             — ratatui TUI (lazygit-style)
    adapters/        — per-provider changelog fetchers
    models.rs        — ChangeEntry, Provider, Subscription
    store.rs         — local SQLite or JSON for subscriptions + cache
    output.rs        — JSON/human/table output formatters
```

## Interfaces

1. **TUI** (`changeloz` or `changeloz tui`) — interactive browse/filter/subscribe
2. **CLI** (`changeloz list`, `changeloz sub anthropic`, `changeloz feed --json`) — scriptable
3. **Future: MCP server** — agents query "what changed in Anthropic this week?"

## Tech

- **Language**: Rust
- **TUI**: ratatui + crossterm
- **HTTP**: reqwest
- **CLI**: clap
- **Storage**: local SQLite (rusqlite) or JSON file
- **Parsing**: quick-xml (RSS/Atom), serde_json

## Open Questions / Spikes Needed

1. What exact data sources exist per provider? (GitHub releases atom, RSS, changelog pages?)
2. OpenAPI spec diffing — is there a Rust crate or do we roll our own?
3. Local storage: SQLite vs flat JSON file? (SQLite probably better for querying)
4. Rate limiting / caching strategy for fetches

## Next Steps

1. Spike: research actual data sources for each LLM provider
2. Scaffold Rust project with clap + ratatui
3. Build first adapter (Anthropic — we know the ecosystem best)
4. Wire up TUI with mock data
