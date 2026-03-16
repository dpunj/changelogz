use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

const CYAN: Color = Color::Rgb(86, 182, 194);
const MUTED: Color = Color::Rgb(90, 90, 90);
const GREEN: Color = Color::Rgb(80, 200, 120);
const YELLOW: Color = Color::Rgb(220, 180, 50);
const BLUE: Color = Color::Rgb(100, 150, 240);
const WHITE: Color = Color::Rgb(220, 220, 220);
const DIM: Color = Color::Rgb(120, 120, 120);
const SURFACE: Color = Color::Rgb(40, 40, 40);

/// Convert a markdown string into styled ratatui Lines
pub fn render_markdown(md: &str) -> Vec<Line<'static>> {
    let options = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES;
    let parser = Parser::new_ext(md, options);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut style_stack: Vec<Style> = vec![Style::default().fg(Color::Rgb(180, 180, 180))];
    let mut list_depth: usize = 0;
    let mut in_code_block = false;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    flush_line(&mut lines, &mut current_spans);
                    let style = match level {
                        pulldown_cmark::HeadingLevel::H1 => {
                            Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
                        }
                        pulldown_cmark::HeadingLevel::H2 => {
                            Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
                        }
                        pulldown_cmark::HeadingLevel::H3 => {
                            Style::default().fg(WHITE).add_modifier(Modifier::BOLD)
                        }
                        _ => Style::default().fg(WHITE),
                    };
                    style_stack.push(style);
                }
                Tag::Paragraph => {
                    // Don't add blank line before first paragraph or inside list items
                    if !lines.is_empty() && list_depth == 0 {
                        flush_line(&mut lines, &mut current_spans);
                    }
                }
                Tag::Emphasis => {
                    let base = current_style(&style_stack);
                    style_stack.push(base.fg(YELLOW).add_modifier(Modifier::ITALIC));
                }
                Tag::Strong => {
                    let base = current_style(&style_stack);
                    style_stack.push(base.fg(WHITE).add_modifier(Modifier::BOLD));
                }
                Tag::Strikethrough => {
                    let base = current_style(&style_stack);
                    style_stack.push(base.fg(DIM).add_modifier(Modifier::CROSSED_OUT));
                }
                Tag::CodeBlock(_) => {
                    flush_line(&mut lines, &mut current_spans);
                    in_code_block = true;
                }
                Tag::List(_) => {
                    if list_depth == 0 {
                        flush_line(&mut lines, &mut current_spans);
                    }
                    list_depth += 1;
                }
                Tag::Item => {
                    flush_line(&mut lines, &mut current_spans);
                    let indent = "  ".repeat(list_depth.saturating_sub(1));
                    let bullet = if list_depth <= 1 { "•" } else { "◦" };
                    current_spans.push(Span::styled(
                        format!("{} {} ", indent, bullet),
                        Style::default().fg(MUTED),
                    ));

                }
                Tag::Link { dest_url, .. } => {
                    style_stack.push(Style::default().fg(BLUE).add_modifier(Modifier::UNDERLINED));
                    // We'll store the URL to maybe append later
                    let _ = dest_url; // URL handling — text content will be the visible part
                }
                Tag::BlockQuote(_) => {
                    flush_line(&mut lines, &mut current_spans);
                    style_stack.push(Style::default().fg(DIM));
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_) => {
                    style_stack.pop();
                    flush_line(&mut lines, &mut current_spans);
                }
                TagEnd::Paragraph => {
                    flush_line(&mut lines, &mut current_spans);
                }
                TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough | TagEnd::Link => {
                    style_stack.pop();
                }
                TagEnd::CodeBlock => {
                    in_code_block = false;
                    flush_line(&mut lines, &mut current_spans);
                }
                TagEnd::List(_) => {
                    list_depth = list_depth.saturating_sub(1);
                    if list_depth == 0 {
                        flush_line(&mut lines, &mut current_spans);
                    }
                }
                TagEnd::Item => {
                    flush_line(&mut lines, &mut current_spans);
                }
                TagEnd::BlockQuote(_) => {
                    style_stack.pop();
                    flush_line(&mut lines, &mut current_spans);
                }
                _ => {}
            },
            Event::Text(text) => {
                if in_code_block {
                    // Render each line of code block with background
                    for line in text.lines() {
                        current_spans.push(Span::styled(
                            format!("  {}", line),
                            Style::default().fg(GREEN).bg(SURFACE),
                        ));
                        flush_line(&mut lines, &mut current_spans);
                    }
                } else {
                    let style = current_style(&style_stack);
                    current_spans.push(Span::styled(text.to_string(), style));
                }
            }
            Event::Code(code) => {
                current_spans.push(Span::styled(
                    format!("`{}`", code),
                    Style::default().fg(GREEN).bg(SURFACE),
                ));
            }
            Event::SoftBreak => {
                current_spans.push(Span::raw(" "));
            }
            Event::HardBreak => {
                flush_line(&mut lines, &mut current_spans);
            }
            Event::Rule => {
                flush_line(&mut lines, &mut current_spans);
                lines.push(Line::from(Span::styled(
                    "─".repeat(40),
                    Style::default().fg(MUTED),
                )));
            }
            _ => {}
        }
    }

    // Flush any remaining spans
    flush_line(&mut lines, &mut current_spans);

    lines
}

fn current_style(stack: &[Style]) -> Style {
    stack.last().copied().unwrap_or_default()
}

fn flush_line(lines: &mut Vec<Line<'static>>, spans: &mut Vec<Span<'static>>) {
    if !spans.is_empty() {
        lines.push(Line::from(spans.drain(..).collect::<Vec<_>>()));
    }
}
