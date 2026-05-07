use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::{App, Focus, Mode};
use crate::editor::Editor;
use crate::nav;
use crate::theme;

const HEADER_HEIGHT: u16 = 5;
const COLUMN_HEADER_HEIGHT: u16 = 1;
const FOOTER_HEIGHT: u16 = 3;
const CHROME_HEIGHT: u16 = HEADER_HEIGHT + COLUMN_HEADER_HEIGHT + FOOTER_HEIGHT;

const ICON_TOP: &str = "▄███▄";
const ICON_MID: &str = "█▒▒▒█";
const ICON_BOT: &str = "▀███▀";

const WORDMARK_TOP: &str = "█▀▀█ █   █ ▄▀▀▀▄ ▄▀▀▀▀ █▀▀█ █   █ ▄▀▀▀▄ █▀▀▀▄";
const WORDMARK_MID: &str = "█▀▀▀ █▀▀▀█ █   █  ▀▀▀▄ █▀▀▀ █▀▀▀█ █   █ █▀█▀ ";
const WORDMARK_BOT: &str = "█    █   █ ▀▄▄▄▀ ▄▄▄▄▀ █    █   █ ▀▄▄▄▀ █  ▀▄";

/// Render the entire UI.
pub fn draw(f: &mut Frame, app: &App, editor: Option<&Editor>) {
    let area = f.area();

    f.render_widget(Clear, area);
    let bg_block = Block::default().style(theme::text());
    f.render_widget(bg_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(HEADER_HEIGHT),
            Constraint::Length(COLUMN_HEADER_HEIGHT),
            Constraint::Min(1),
            Constraint::Length(FOOTER_HEIGHT),
        ])
        .split(area);

    draw_header(f, app, chunks[0]);
    draw_column_headers(f, chunks[1]);

    if let Some(ed) = editor {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[2]);
        draw_file_list(f, app, split[0]);
        draw_editor_panel(f, ed, split[1], app.focus == Focus::Right);
    } else {
        draw_file_list(f, app, chunks[2]);
    }

    draw_footer(f, app, chunks[3]);
}

/// Compute (cols, rows) for the embedded editor given the full terminal area.
/// Mirrors the layout split in `draw` so the pty is sized to match the panel.
pub fn editor_panel_size(area: Rect) -> (u16, u16) {
    let main_height = area.height.saturating_sub(CHROME_HEIGHT);
    let right_width = area.width / 2;
    // -1 col for the left border of the editor panel.
    (right_width.saturating_sub(1), main_height)
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let lines = vec![
        Line::from(vec![
            Span::styled(format!(" {}  ", ICON_TOP), theme::title()),
            Span::styled(WORDMARK_TOP, theme::title()),
        ]),
        Line::from(vec![
            Span::styled(format!(" {}  ", ICON_MID), theme::title()),
            Span::styled(WORDMARK_MID, theme::title()),
            Span::styled("  │ ", theme::header_bg()),
            Span::styled(app.display_path(), theme::header_bg()),
        ]),
        Line::from(vec![
            Span::styled(format!(" {}  ", ICON_BOT), theme::title()),
            Span::styled(WORDMARK_BOT, theme::title()),
        ]),
    ];

    let header = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(theme::border())
            .style(theme::header_bg()),
    );
    f.render_widget(header, area);
}

fn draw_column_headers(f: &mut Frame, area: Rect) {
    let width = area.width as usize;
    let size_label = "Size";
    let name_label = "Name";
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

            let available = inner.width as usize;
            let size_width = size_str.len();
            let name_width = available.saturating_sub(size_width + 4);

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

fn draw_editor_panel(f: &mut Frame, editor: &Editor, area: Rect, focused: bool) {
    let border_style = if focused {
        Style::default().fg(theme::PHOSPHOR).bg(theme::BG).add_modifier(Modifier::BOLD)
    } else {
        theme::border()
    };
    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(border_style)
        .style(theme::text());

    let inner = block.inner(area);
    f.render_widget(block, area);

    let parser = editor.parser();
    let parser = match parser.lock() {
        Ok(p) => p,
        Err(_) => return,
    };
    let screen = parser.screen();

    let mut lines: Vec<Line> = Vec::with_capacity(inner.height as usize);
    for row in 0..inner.height {
        let mut spans: Vec<Span> = Vec::new();
        let mut current_text = String::new();
        let mut current_style = Style::default();
        let mut have_run = false;

        for col in 0..inner.width {
            let (text, style) = match screen.cell(row, col) {
                Some(cell) => {
                    let s = cell.contents().to_string();
                    let display = if s.is_empty() { " ".to_string() } else { s };
                    (display, cell_style(cell))
                }
                None => (" ".to_string(), Style::default().fg(theme::TEXT_PRIMARY).bg(theme::BG)),
            };

            if have_run && style != current_style {
                spans.push(Span::styled(std::mem::take(&mut current_text), current_style));
            }
            current_style = style;
            current_text.push_str(&text);
            have_run = true;
        }
        if have_run {
            spans.push(Span::styled(current_text, current_style));
        }
        lines.push(Line::from(spans));
    }

    let para = Paragraph::new(lines);
    f.render_widget(para, inner);

    if focused {
        let (cy, cx) = screen.cursor_position();
        let abs_x = inner.x + cx;
        let abs_y = inner.y + cy;
        if abs_x < inner.x + inner.width && abs_y < inner.y + inner.height {
            f.set_cursor_position((abs_x, abs_y));
        }
    }
}

fn cell_style(cell: &vt100::Cell) -> Style {
    let mut style = Style::default()
        .fg(map_fg(cell.fgcolor()))
        .bg(map_bg(cell.bgcolor()));
    if cell.bold() {
        style = style.add_modifier(Modifier::BOLD);
    }
    if cell.italic() {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if cell.underline() {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    if cell.inverse() {
        style = style.add_modifier(Modifier::REVERSED);
    }
    style
}

fn map_fg(c: vt100::Color) -> Color {
    match c {
        vt100::Color::Default => theme::TEXT_PRIMARY,
        vt100::Color::Idx(i) => Color::Indexed(i),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}

fn map_bg(c: vt100::Color) -> Color {
    match c {
        vt100::Color::Default => theme::BG,
        vt100::Color::Idx(i) => Color::Indexed(i),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}

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
            if let Some(ref err) = app.status_message {
                Line::from(vec![Span::styled(
                    format!("  \u{26a0} {}", err),
                    theme::error(),
                )])
            } else {
                let dirs = app.dir_count();
                let files = app.file_count();
                let hidden_indicator = if app.show_hidden { " [H]" } else { "" };
                let focus_indicator = if app.editor_alive {
                    match app.focus {
                        Focus::Left => " [browse]",
                        Focus::Right => " [vim]",
                    }
                } else {
                    ""
                };

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
                    Span::styled("<C-Space>", theme::key_hint()),
                    Span::styled("focus ", theme::status()),
                    Span::styled("<q>", theme::key_hint()),
                    Span::styled("quit", theme::status()),
                    Span::styled(
                        format!(
                            "  │ {} dirs, {} files{}{}",
                            dirs, files, hidden_indicator, focus_indicator
                        ),
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
