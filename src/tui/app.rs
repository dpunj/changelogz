use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::sync::mpsc;
use std::time::Duration;

use crate::adapters;
use crate::models::{ChangeEntry, ChangeKind, Provider};
use crate::store::Store;

use super::ui;

#[derive(PartialEq)]
pub enum Panel {
    Providers,
    Feed,
    Detail,
}

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Search,
}

pub struct App {
    pub providers: Vec<(Provider, bool)>,
    pub entries: Vec<ChangeEntry>,
    pub filtered_entries: Vec<ChangeEntry>,
    pub provider_index: usize,
    pub feed_index: usize,
    pub active_panel: Panel,
    pub filter_kind: Option<ChangeKind>,
    pub scroll_offset: u16,
    pub should_quit: bool,
    pub status_msg: String,
    pub input_mode: InputMode,
    pub search_query: String,
    pub is_fetching: bool,
}

impl App {
    pub fn new(store: &Store) -> Result<Self> {
        let subscribed = store.subscriptions()?;
        let providers: Vec<(Provider, bool)> = Provider::all()
            .into_iter()
            .map(|p| {
                let is_sub = subscribed.contains(&p);
                (p, is_sub)
            })
            .collect();

        let entries = store.get_feed(None, None, 500)?;

        let mut app = App {
            providers,
            entries: entries.clone(),
            filtered_entries: entries,
            provider_index: 0,
            feed_index: 0,
            active_panel: Panel::Feed,
            filter_kind: None,
            scroll_offset: 0,
            should_quit: false,
            status_msg: String::new(),
            input_mode: InputMode::Normal,
            search_query: String::new(),
            is_fetching: false,
        };
        app.apply_filter();
        Ok(app)
    }

    pub fn selected_entry(&self) -> Option<&ChangeEntry> {
        self.filtered_entries.get(self.feed_index)
    }

    pub fn apply_filter(&mut self) {
        let query = self.search_query.to_lowercase();
        self.filtered_entries = self
            .entries
            .iter()
            .filter(|e| {
                if let Some(ref kind) = self.filter_kind {
                    if &e.kind != kind {
                        return false;
                    }
                }
                if !query.is_empty() {
                    let haystack = format!(
                        "{} {} {} {}",
                        e.title, e.body, e.provider, e.tags.join(" ")
                    )
                    .to_lowercase();
                    if !haystack.contains(&query) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        if self.feed_index >= self.filtered_entries.len() {
            self.feed_index = self.filtered_entries.len().saturating_sub(1);
        }
        self.scroll_offset = 0;
    }

    fn toggle_subscription(&mut self, store: &Store) -> Result<()> {
        if let Some((provider, subscribed)) = self.providers.get_mut(self.provider_index) {
            if *subscribed {
                store.unsubscribe(provider)?;
                *subscribed = false;
                self.status_msg = format!("Unsubscribed from {}", provider);
            } else {
                store.subscribe(provider)?;
                *subscribed = true;
                self.status_msg = format!("Subscribed to {}", provider);
            }
        }
        Ok(())
    }

    fn open_selected_url(&self) {
        if let Some(entry) = self.selected_entry() {
            let _ = open::that(&entry.url);
        }
    }
}

/// Message from background fetch thread
enum FetchMsg {
    Progress(String),
    Done(Vec<ChangeEntry>),
    Error(String),
}

pub fn run_tui() -> Result<()> {
    let store = Store::open()?;
    let mut app = App::new(&store)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut app, &store);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    store: &Store,
) -> Result<()> {
    let (fetch_tx, fetch_rx) = mpsc::channel::<FetchMsg>();

    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        // Check for completed fetch results
        if let Ok(msg) = fetch_rx.try_recv() {
            match msg {
                FetchMsg::Progress(status) => {
                    app.status_msg = status;
                }
                FetchMsg::Done(entries) => {
                    let count = store.upsert_entries(&entries).unwrap_or(0);
                    app.entries = store.get_feed(None, None, 500)?;
                    app.apply_filter();
                    app.is_fetching = false;
                    app.status_msg = format!("Fetched {} new entries", count);
                }
                FetchMsg::Error(err) => {
                    app.is_fetching = false;
                    app.status_msg = format!("Fetch error: {}", err);
                }
            }
        }

        // Poll for keyboard events with timeout so we can check fetch results
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Search => handle_search_input(app, key),
                    InputMode::Normal => handle_normal_input(app, store, key, &fetch_tx)?,
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_search_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.search_query.clear();
            app.apply_filter();
            app.status_msg.clear();
        }
        KeyCode::Enter => {
            app.input_mode = InputMode::Normal;
            app.apply_filter();
            app.status_msg = if app.search_query.is_empty() {
                String::new()
            } else {
                format!(
                    "Search: \"{}\" ({} results)",
                    app.search_query,
                    app.filtered_entries.len()
                )
            };
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            app.apply_filter();
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
            app.apply_filter();
        }
        _ => {}
    }
}

fn handle_normal_input(
    app: &mut App,
    store: &Store,
    key: KeyEvent,
    fetch_tx: &mpsc::Sender<FetchMsg>,
) -> Result<()> {
    match key.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        KeyCode::Tab => {
            app.active_panel = match app.active_panel {
                Panel::Providers => Panel::Feed,
                Panel::Feed => Panel::Detail,
                Panel::Detail => Panel::Providers,
            };
            app.scroll_offset = 0;
        }
        KeyCode::BackTab => {
            app.active_panel = match app.active_panel {
                Panel::Providers => Panel::Detail,
                Panel::Feed => Panel::Providers,
                Panel::Detail => Panel::Feed,
            };
            app.scroll_offset = 0;
        }
        KeyCode::Char('j') | KeyCode::Down => match app.active_panel {
            Panel::Providers => {
                if app.provider_index < app.providers.len().saturating_sub(1) {
                    app.provider_index += 1;
                }
            }
            Panel::Feed => {
                if app.feed_index < app.filtered_entries.len().saturating_sub(1) {
                    app.feed_index += 1;
                    app.scroll_offset = 0;
                }
            }
            Panel::Detail => {
                app.scroll_offset = app.scroll_offset.saturating_add(1);
            }
        },
        KeyCode::Char('k') | KeyCode::Up => match app.active_panel {
            Panel::Providers => {
                app.provider_index = app.provider_index.saturating_sub(1);
            }
            Panel::Feed => {
                app.feed_index = app.feed_index.saturating_sub(1);
                app.scroll_offset = 0;
            }
            Panel::Detail => {
                app.scroll_offset = app.scroll_offset.saturating_sub(1);
            }
        },
        KeyCode::Char('g') => {
            // Jump to top
            match app.active_panel {
                Panel::Feed => {
                    app.feed_index = 0;
                    app.scroll_offset = 0;
                }
                Panel::Providers => app.provider_index = 0,
                Panel::Detail => app.scroll_offset = 0,
            }
        }
        KeyCode::Char('G') => {
            // Jump to bottom
            match app.active_panel {
                Panel::Feed => {
                    app.feed_index = app.filtered_entries.len().saturating_sub(1);
                    app.scroll_offset = 0;
                }
                Panel::Providers => {
                    app.provider_index = app.providers.len().saturating_sub(1);
                }
                _ => {}
            }
        }
        KeyCode::Enter => {
            if app.active_panel == Panel::Providers {
                app.toggle_subscription(store)?;
            }
        }
        // Open URL in browser
        KeyCode::Char('o') => {
            app.open_selected_url();
            if let Some(entry) = app.selected_entry() {
                app.status_msg = format!("Opened: {}", entry.url);
            }
        }
        // Search
        KeyCode::Char('/') => {
            app.input_mode = InputMode::Search;
            app.search_query.clear();
            app.active_panel = Panel::Feed;
        }
        // Clear search
        KeyCode::Esc => {
            if !app.search_query.is_empty() {
                app.search_query.clear();
                app.apply_filter();
                app.status_msg.clear();
            }
        }
        // Fetch from providers
        KeyCode::Char('r') => {
            if !app.is_fetching {
                app.is_fetching = true;
                app.status_msg = "Fetching from providers...".to_string();

                let subscribed: Vec<Provider> = app
                    .providers
                    .iter()
                    .filter(|(_, sub)| *sub)
                    .map(|(p, _)| p.clone())
                    .collect();

                if subscribed.is_empty() {
                    app.is_fetching = false;
                    app.status_msg = "No subscriptions — subscribe first".to_string();
                } else {
                    let tx = fetch_tx.clone();
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        let mut all_entries = Vec::new();

                        for provider in &subscribed {
                            let _ = tx.send(FetchMsg::Progress(format!(
                                "Fetching {}...",
                                provider
                            )));
                            let adapter = adapters::adapter_for(provider);
                            match rt.block_on(adapter.fetch()) {
                                Ok(entries) => {
                                    all_entries.extend(entries);
                                }
                                Err(e) => {
                                    let _ = tx.send(FetchMsg::Error(format!(
                                        "{}: {}",
                                        provider, e
                                    )));
                                    return;
                                }
                            }
                        }

                        let _ = tx.send(FetchMsg::Done(all_entries));
                    });
                }
            }
        }
        // Filter by kind
        KeyCode::Char('1') => {
            app.filter_kind = Some(ChangeKind::Breaking);
            app.apply_filter();
        }
        KeyCode::Char('2') => {
            app.filter_kind = Some(ChangeKind::Deprecation);
            app.apply_filter();
        }
        KeyCode::Char('3') => {
            app.filter_kind = Some(ChangeKind::Feature);
            app.apply_filter();
        }
        KeyCode::Char('4') => {
            app.filter_kind = Some(ChangeKind::ModelRelease);
            app.apply_filter();
        }
        KeyCode::Char('5') => {
            app.filter_kind = Some(ChangeKind::Fix);
            app.apply_filter();
        }
        KeyCode::Char('0') => {
            app.filter_kind = None;
            app.apply_filter();
        }
        _ => {}
    }
    Ok(())
}
