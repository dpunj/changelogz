use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::app::{App, Panel};
use crate::models::ChangeKind;

const CYAN: Color = Color::Rgb(86, 182, 194);
const MUTED: Color = Color::Rgb(90, 90, 90);
const SURFACE: Color = Color::Rgb(30, 30, 30);
const GREEN: Color = Color::Rgb(80, 200, 120);
const RED: Color = Color::Rgb(220, 80, 80);
const YELLOW: Color = Color::Rgb(220, 180, 50);
const MAGENTA: Color = Color::Rgb(180, 120, 220);
const BLUE: Color = Color::Rgb(100, 150, 240);
const WHITE: Color = Color::Rgb(220, 220, 220);
const DIM: Color = Color::Rgb(120, 120, 120);

fn kind_color(kind: &ChangeKind) -> Color {
    match kind {
        ChangeKind::Breaking => RED,
        ChangeKind::Deprecation => YELLOW,
        ChangeKind::Feature => GREEN,
        ChangeKind::ModelRelease => MAGENTA,
        ChangeKind::Fix => BLUE,
        ChangeKind::Other => DIM,
    }
}

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(10),  // main
            Constraint::Length(1), // status
        ])
        .split(f.area());

    draw_header(f, chunks[0]);
    draw_main(f, app, chunks[1]);
    draw_status(f, app, chunks[2]);
}

fn draw_header(f: &mut Frame, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" changeloz", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
        Span::styled("  LLM API changelog tracker", Style::default().fg(MUTED)),
    ]));
    f.render_widget(header, area);
}

fn draw_main(f: &mut Frame, app: &App, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(18),    // providers sidebar
            Constraint::Percentage(45), // feed
            Constraint::Min(30),       // detail
        ])
        .split(area);

    draw_providers(f, app, main_chunks[0]);
    draw_feed(f, app, main_chunks[1]);
    draw_detail(f, app, main_chunks[2]);
}

fn panel_border(active: bool) -> Style {
    if active {
        Style::default().fg(CYAN)
    } else {
        Style::default().fg(MUTED)
    }
}

fn draw_providers(f: &mut Frame, app: &App, area: Rect) {
    let active = app.active_panel == Panel::Providers;

    let items: Vec<ListItem> = app
        .providers
        .iter()
        .enumerate()
        .map(|(i, (provider, subscribed))| {
            let (marker, marker_color) = if *subscribed {
                ("●", GREEN)
            } else {
                ("○", MUTED)
            };

            let selected = i == app.provider_index && active;
            let name_style = if selected {
                Style::default().bg(SURFACE).fg(WHITE).add_modifier(Modifier::BOLD)
            } else if *subscribed {
                Style::default().fg(WHITE)
            } else {
                Style::default().fg(DIM)
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", marker), Style::default().fg(marker_color)),
                Span::styled(provider.to_string(), name_style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(Span::styled(" Providers ", Style::default().fg(if active { CYAN } else { DIM })))
            .borders(Borders::ALL)
            .border_style(panel_border(active)),
    );
    f.render_widget(list, area);
}

fn draw_feed(f: &mut Frame, app: &App, area: Rect) {
    let active = app.active_panel == Panel::Feed;

    let title = match &app.filter_kind {
        Some(k) => format!(" Feed [{}] ", k),
        None => " Feed ".to_string(),
    };

    if app.filtered_entries.is_empty() {
        let empty = Paragraph::new(Text::from(vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled("  No entries yet", Style::default().fg(DIM))),
            Line::from(""),
            Line::from(Span::styled("  Subscribe to providers and", Style::default().fg(MUTED))),
            Line::from(Span::styled("  run `changeloz fetch`", Style::default().fg(CYAN))),
        ]))
        .block(
            Block::default()
                .title(Span::styled(&*title, Style::default().fg(if active { CYAN } else { DIM })))
                .borders(Borders::ALL)
                .border_style(panel_border(active)),
        );
        f.render_widget(empty, area);
        return;
    }

    // Calculate available width for title (area width minus date, provider, kind, padding)
    let fixed_cols = 6 + 5 + 7 + 2; // "MM/DD " + "PRV  " + "KIND   " + padding
    let title_width = (area.width as usize).saturating_sub(fixed_cols + 2); // -2 for borders

    let items: Vec<ListItem> = app
        .filtered_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let selected = i == app.feed_index && active;
            let bg = if selected { SURFACE } else { Color::Reset };

            let truncated_title = if entry.title.len() > title_width && title_width > 3 {
                format!("{}…", &entry.title[..title_width - 1])
            } else {
                entry.title.clone()
            };

            let kind_label: String = format!("{}", entry.kind)
                .chars()
                .take(5)
                .collect();

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{} ", entry.date.format("%m/%d")),
                    Style::default().fg(MUTED).bg(bg),
                ),
                Span::styled(
                    format!("{:<4} ", &entry.provider.id()[..3].to_uppercase()),
                    Style::default().fg(CYAN).bg(bg),
                ),
                Span::styled(
                    format!("{:<6} ", kind_label),
                    Style::default().fg(kind_color(&entry.kind)).bg(bg),
                ),
                Span::styled(
                    truncated_title,
                    Style::default().fg(if selected { WHITE } else { Color::Rgb(180, 180, 180) }).bg(bg),
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(Span::styled(&*title, Style::default().fg(if active { CYAN } else { DIM })))
            .borders(Borders::ALL)
            .border_style(panel_border(active)),
    );
    f.render_widget(list, area);
}

fn draw_detail(f: &mut Frame, app: &App, area: Rect) {
    let active = app.active_panel == Panel::Detail;

    let content = if let Some(entry) = app.selected_entry() {
        let mut lines = vec![
            Line::from(Span::styled(
                &entry.title,
                Style::default().fg(WHITE).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled(entry.provider.to_string(), Style::default().fg(CYAN)),
                Span::styled("  ", Style::default()),
                Span::styled(entry.date.to_string(), Style::default().fg(YELLOW)),
                Span::styled("  ", Style::default()),
                Span::styled(
                    format!("{}", entry.kind),
                    Style::default().fg(kind_color(&entry.kind)),
                ),
            ]),
            Line::from(Span::styled(&entry.url, Style::default().fg(BLUE))),
            Line::from(""),
        ];

        if !entry.tags.is_empty() {
            lines.push(Line::from(
                entry
                    .tags
                    .iter()
                    .map(|t| Span::styled(format!(" {} ", t), Style::default().fg(MUTED)))
                    .collect::<Vec<_>>(),
            ));
            lines.push(Line::from(""));
        }

        // Render body as proper markdown
        let md_lines = super::markdown::render_markdown(&entry.body);
        lines.extend(md_lines);

        Text::from(lines)
    } else {
        Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Select an entry to view details",
                Style::default().fg(MUTED),
            )),
        ])
    };

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .title(Span::styled(" Detail ", Style::default().fg(if active { CYAN } else { DIM })))
                .borders(Borders::ALL)
                .border_style(panel_border(active)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset, 0));

    f.render_widget(paragraph, area);
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let keyhints = vec![
        ("q", "quit"),
        ("Tab", "panel"),
        ("j/k", "nav"),
        ("Enter", "toggle"),
        ("1-5", "filter"),
        ("0", "clear"),
        ("r", "refresh"),
    ];

    let mut spans: Vec<Span> = vec![Span::styled(" ", Style::default())];
    for (i, (key, desc)) in keyhints.iter().enumerate() {
        spans.push(Span::styled(*key, Style::default().fg(WHITE).add_modifier(Modifier::BOLD)));
        spans.push(Span::styled(format!(" {}", desc), Style::default().fg(MUTED)));
        if i < keyhints.len() - 1 {
            spans.push(Span::styled("  ", Style::default()));
        }
    }

    // Append status message if non-default
    if !app.status_msg.is_empty() && !app.status_msg.starts_with("q:") {
        spans.push(Span::styled("  │  ", Style::default().fg(MUTED)));
        spans.push(Span::styled(&app.status_msg, Style::default().fg(CYAN)));
    }

    let status = Paragraph::new(Line::from(spans));
    f.render_widget(status, area);
}
