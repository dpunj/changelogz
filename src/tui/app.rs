use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use crate::models::{ChangeEntry, ChangeKind, Provider};
use crate::store::Store;

use super::ui;

#[derive(PartialEq)]
pub enum Panel {
    Providers,
    Feed,
    Detail,
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
            status_msg: String::from("q: quit | Tab: switch panel | Enter: toggle sub | 1-5: filter kind | 0: clear filter | r: refresh"),
        };
        app.apply_filter();
        Ok(app)
    }

    pub fn selected_entry(&self) -> Option<&ChangeEntry> {
        self.filtered_entries.get(self.feed_index)
    }

    pub fn apply_filter(&mut self) {
        self.filtered_entries = self.entries.iter().filter(|e| {
            // Filter by kind
            if let Some(ref kind) = self.filter_kind {
                if &e.kind != kind {
                    return false;
                }
            }
            true
        }).cloned().collect();

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
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) || key.code == KeyCode::Char('q') => {
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
                KeyCode::Enter => {
                    if app.active_panel == Panel::Providers {
                        app.toggle_subscription(store)?;
                    }
                }
                KeyCode::Char('r') => {
                    app.status_msg = "Refreshing...".to_string();
                    // We'll do async fetch in a separate step
                    // For now reload from store
                    app.entries = store.get_feed(None, None, 500)?;
                    app.apply_filter();
                    app.status_msg = format!("Loaded {} entries", app.entries.len());
                }
                // Filter by kind
                KeyCode::Char('1') => { app.filter_kind = Some(ChangeKind::Breaking); app.apply_filter(); }
                KeyCode::Char('2') => { app.filter_kind = Some(ChangeKind::Deprecation); app.apply_filter(); }
                KeyCode::Char('3') => { app.filter_kind = Some(ChangeKind::Feature); app.apply_filter(); }
                KeyCode::Char('4') => { app.filter_kind = Some(ChangeKind::ModelRelease); app.apply_filter(); }
                KeyCode::Char('5') => { app.filter_kind = Some(ChangeKind::Fix); app.apply_filter(); }
                KeyCode::Char('0') => { app.filter_kind = None; app.apply_filter(); }
                _ => {}
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
