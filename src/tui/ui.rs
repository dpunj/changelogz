use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::app::{App, Panel};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Min(10),   // main
            Constraint::Length(2), // status bar
        ])
        .split(f.area());

    draw_header(f, chunks[0]);
    draw_main(f, app, chunks[1]);
    draw_status(f, app, chunks[2]);
}

fn draw_header(f: &mut Frame, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" changeloz ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("— LLM API changelog tracker", Style::default().fg(Color::DarkGray)),
    ]))
    .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(header, area);
}

fn draw_main(f: &mut Frame, app: &App, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20),   // providers
            Constraint::Percentage(40), // feed
            Constraint::Percentage(40), // detail
        ])
        .split(area);

    draw_providers(f, app, main_chunks[0]);
    draw_feed(f, app, main_chunks[1]);
    draw_detail(f, app, main_chunks[2]);
}

fn draw_providers(f: &mut Frame, app: &App, area: Rect) {
    let border_color = if app.active_panel == Panel::Providers {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let items: Vec<ListItem> = app
        .providers
        .iter()
        .enumerate()
        .map(|(i, (provider, subscribed))| {
            let marker = if *subscribed { "●" } else { "○" };
            let color = if *subscribed { Color::Green } else { Color::DarkGray };
            let style = if i == app.provider_index && app.active_panel == Panel::Providers {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", marker), Style::default().fg(color)),
                Span::styled(provider.to_string(), style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Providers ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(list, area);
}

fn draw_feed(f: &mut Frame, app: &App, area: Rect) {
    let border_color = if app.active_panel == Panel::Feed {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let filter_label = match &app.filter_kind {
        Some(k) => format!(" Feed [{}] ", k),
        None => " Feed ".to_string(),
    };

    let items: Vec<ListItem> = app
        .filtered_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let style = if i == app.feed_index && app.active_panel == Panel::Feed {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            let kind_color = match entry.kind {
                crate::models::ChangeKind::Breaking => Color::Red,
                crate::models::ChangeKind::Deprecation => Color::Yellow,
                crate::models::ChangeKind::Feature => Color::Green,
                crate::models::ChangeKind::ModelRelease => Color::Magenta,
                crate::models::ChangeKind::Fix => Color::Blue,
                crate::models::ChangeKind::Other => Color::DarkGray,
            };

            let truncated_title = if entry.title.len() > 50 {
                format!("{}...", &entry.title[..47])
            } else {
                entry.title.clone()
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", entry.date.format("%m/%d")), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:<4} ", &entry.provider.id()[..3].to_uppercase()), Style::default().fg(Color::Cyan)),
                Span::styled(format!("{:<6} ", format!("{}", entry.kind).chars().take(5).collect::<String>()), Style::default().fg(kind_color)),
                Span::styled(truncated_title, style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(filter_label)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(list, area);
}

fn draw_detail(f: &mut Frame, app: &App, area: Rect) {
    let border_color = if app.active_panel == Panel::Detail {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let content = if let Some(entry) = app.selected_entry() {
        let mut lines = vec![
            Line::from(vec![
                Span::styled(&entry.title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Provider: ", Style::default().fg(Color::DarkGray)),
                Span::styled(entry.provider.to_string(), Style::default().fg(Color::Cyan)),
                Span::styled("  Date: ", Style::default().fg(Color::DarkGray)),
                Span::styled(entry.date.to_string(), Style::default().fg(Color::Yellow)),
                Span::styled("  Kind: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{}", entry.kind), Style::default().fg(Color::Magenta)),
            ]),
            Line::from(vec![
                Span::styled("URL: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&entry.url, Style::default().fg(Color::Blue)),
            ]),
            Line::from(""),
        ];

        for line in entry.body.lines() {
            lines.push(Line::from(Span::raw(line)));
        }

        Text::from(lines)
    } else {
        Text::from(Span::styled(
            "No entry selected",
            Style::default().fg(Color::DarkGray),
        ))
    };

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .title(" Detail ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset, 0));

    f.render_widget(paragraph, area);
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(&app.status_msg, Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(status, area);
}
