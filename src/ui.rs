use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::{App, Mode};
use crate::nav;
use crate::theme;

/// Render the entire UI.
pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();

    // Fill background
    f.render_widget(Clear, area);
    let bg_block = Block::default().style(theme::text());
    f.render_widget(bg_block, area);

    // Main layout: header (3), column headers (1), file list (fill), footer (3)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    draw_header(f, app, chunks[0]);
    draw_column_headers(f, chunks[1]);
    draw_file_list(f, app, chunks[2]);
    draw_footer(f, app, chunks[3]);
}

/// Header: k9s-style crumb bar with app name and path.
fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled("  PHOSPHOR", theme::title()),
        Span::styled(" │ ", theme::header_bg()),
        Span::styled(app.display_path(), theme::header_bg()),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(theme::border())
            .style(theme::header_bg()),
    );
    f.render_widget(header, area);
}

/// Column header row showing Name and Size labels.
fn draw_column_headers(f: &mut Frame, area: Rect) {
    let width = area.width as usize;
    let size_label = "Size";
    let name_label = "Name";
    // 4 chars prefix (▐ + space or just spaces) + 2 gap + size label
    let name_width = width.saturating_sub(size_label.len() + 5);
    let line_text = format!(
        "  {:<nw$}  {}",
        name_label,
        size_label,
        nw = name_width,
    );
    let para = Paragraph::new(Span::styled(line_text, theme::column_header()));
    f.render_widget(para, area);
}

/// Scrollable file list.
fn draw_file_list(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(theme::border())
        .style(theme::text());

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.visible.is_empty() {
        let msg = if app.entries.is_empty() {
            "  (empty directory)"
        } else {
            "  (no matches)"
        };
        let para = Paragraph::new(Span::styled(msg, theme::text_dim()));
        f.render_widget(para, inner);
        return;
    }

    // Calculate scroll offset to keep cursor visible
    let list_height = inner.height as usize;
    let offset = if app.cursor >= list_height {
        app.cursor - list_height + 1
    } else {
        0
    };

    let items: Vec<ListItem> = app
        .visible
        .iter()
        .enumerate()
        .skip(offset)
        .take(list_height)
        .map(|(view_idx, &entry_idx)| {
            let entry = &app.entries[entry_idx];
            let is_selected = view_idx == app.cursor;
            let is_odd = view_idx % 2 == 1;

            let display_name = if entry.is_dir {
                format!("{}/", entry.name)
            } else {
                entry.name.clone()
            };

            let size_str = if entry.is_dir {
                String::new()
            } else {
                nav::format_size(entry.size)
            };

            // Pad the name to fill available width, then append size
            let available = inner.width as usize;
            let size_width = size_str.len();
            let name_width = available.saturating_sub(size_width + 4); // 2 prefix + 2 gap

            let prefix = if is_selected { "\u{2590} " } else { "  " };
            let padded_name = if display_name.len() > name_width {
                display_name[..name_width].to_string()
            } else {
                format!("{:<width$}", display_name, width = name_width)
            };

            let line_text = format!("{}{}  {}", prefix, padded_name, size_str);

            let style = if is_selected {
                if entry.is_dir {
                    theme::selected_dir()
                } else {
                    theme::selected()
                }
            } else if entry.is_dir {
                if is_odd { theme::dir_entry_dim() } else { theme::dir_entry() }
            } else if is_odd {
                theme::text_dim()
            } else {
                theme::text()
            };

            ListItem::new(Span::styled(line_text, style))
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, inner);
}

/// Footer: status bar or search input with k9s-style key hints.
fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let content = match &app.mode {
        Mode::Search => {
            let query_display = format!("  /{}\u{2588}", app.search_query);
            Line::from(vec![
                Span::styled(query_display, theme::search_input()),
                Span::styled(
                    format!("  {} matches", app.visible.len()),
                    theme::status(),
                ),
            ])
        }
        Mode::Normal => {
            // Show error if present
            if let Some(ref err) = app.status_message {
                Line::from(vec![Span::styled(
                    format!("  \u{26a0} {}", err),
                    theme::error(),
                )])
            } else {
                let dirs = app.dir_count();
                let files = app.file_count();
                let hidden_indicator = if app.show_hidden { " [H]" } else { "" };

                let filter_indicator = if !app.search_query.is_empty() {
                    format!("  filter: \"{}\"", app.search_query)
                } else {
                    String::new()
                };

                let mut spans = vec![
                    Span::styled("  ", theme::status()),
                    Span::styled("</>", theme::key_hint()),
                    Span::styled("search ", theme::status()),
                    Span::styled("<.>", theme::key_hint()),
                    Span::styled("hidden ", theme::status()),
                    Span::styled("<q>", theme::key_hint()),
                    Span::styled("quit", theme::status()),
                    Span::styled(
                        format!("  │ {} dirs, {} files{}", dirs, files, hidden_indicator),
                        theme::status(),
                    ),
                ];

                if !filter_indicator.is_empty() {
                    spans.push(Span::styled(filter_indicator, theme::status()));
                }

                Line::from(spans)
            }
        }
    };

    let footer = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(theme::border())
            .style(theme::text()),
    );
    f.render_widget(footer, area);
}
