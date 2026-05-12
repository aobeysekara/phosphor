use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, Focus, Mode};
use crate::editor::Editor;
use crate::md::{self, MdViewer};
use crate::nav;
use crate::theme;
use crate::tree;

const HEADER_HEIGHT: u16 = 5;
const COLUMN_HEADER_HEIGHT: u16 = 1;
const METADATA_HEIGHT: u16 = 3;
const FOOTER_HEIGHT: u16 = 3;
const CHROME_HEIGHT: u16 =
    HEADER_HEIGHT + COLUMN_HEADER_HEIGHT + METADATA_HEIGHT + FOOTER_HEIGHT;

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
            Constraint::Length(METADATA_HEIGHT),
            Constraint::Length(FOOTER_HEIGHT),
        ])
        .split(area);

    draw_header(f, app, chunks[0]);
    draw_column_headers(f, chunks[1]);

    let main_area = chunks[2];
    let right_active = editor.is_some() || app.md_viewer.is_some();

    if right_active {
        let tree_pct = app.tree_pct;
        let right_pct = app.right_pct;
        let list_pct = 100u16.saturating_sub(tree_pct).saturating_sub(right_pct);
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(tree_pct),
                Constraint::Percentage(list_pct),
                Constraint::Percentage(right_pct),
            ])
            .split(main_area);
        draw_tree(f, app, split[0]);
        draw_file_list(f, app, split[1]);
        let focused = app.focus == Focus::Right;
        if let Some(ed) = editor {
            draw_editor_panel(f, ed, split[2], focused);
        } else if let Some(v) = &app.md_viewer {
            draw_md_viewer(f, v, split[2], focused);
        }
    } else {
        let tree_pct = app.tree_pct;
        let list_pct = 100u16.saturating_sub(tree_pct);
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(tree_pct),
                Constraint::Percentage(list_pct),
            ])
            .split(main_area);
        draw_tree(f, app, split[0]);
        draw_file_list(f, app, split[1]);
    }

    draw_metadata(f, app, chunks[3]);
    draw_footer(f, app, chunks[4]);
}

/// Compute (cols, rows) for the embedded editor given the full terminal area
/// and the current `right_pct` from the app. Mirrors the layout split in `draw`
/// so the pty is sized to match the panel.
pub fn right_panel_size(area: Rect, right_pct: u16) -> (u16, u16) {
    let main_height = area.height.saturating_sub(CHROME_HEIGHT);
    let editor_width = area.width.saturating_mul(right_pct) / 100;
    (editor_width.saturating_sub(1), main_height)
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

fn draw_tree(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(theme::border())
        .style(theme::text());

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.tree.is_empty() || inner.width == 0 {
        return;
    }

    let max_width = inner.width as usize;
    let visible_rows = inner.height as usize;

    let style = Style::default().fg(theme::ORANGE).bg(theme::BG);

    let lines: Vec<Line> = app
        .tree
        .iter()
        .take(visible_rows)
        .map(|node| Line::from(Span::styled(tree::render_line(node, max_width), style)))
        .collect();

    let para = Paragraph::new(lines);
    f.render_widget(para, inner);
}

fn draw_metadata(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(theme::border())
        .style(theme::text());

    let inner_para = match app.selected_entry() {
        None => Paragraph::new(Span::styled("  (nothing selected)", theme::text_dim())),
        Some(entry) => {
            let name_span = if entry.is_dir {
                Span::styled(format!("  {}/", entry.name), theme::dir_entry())
            } else {
                Span::styled(format!("  {}", entry.name), theme::text())
            };

            let mut spans = vec![name_span];

            if !entry.is_dir {
                spans.push(Span::styled("  ·  ", theme::text_dim()));
                spans.push(Span::styled(nav::format_size(entry.size), theme::status()));
            }

            if let Ok(metadata) = std::fs::metadata(&entry.path) {
                spans.push(Span::styled("  ·  ", theme::text_dim()));
                spans.push(Span::styled(nav::format_perms(&metadata), theme::status()));

                if let Ok(modified) = metadata.modified() {
                    spans.push(Span::styled("  ·  ", theme::text_dim()));
                    spans.push(Span::styled(nav::format_modified(modified), theme::status()));
                }
            }

            spans.push(Span::styled("  ·  ", theme::text_dim()));
            let type_label = if entry.is_dir {
                "directory"
            } else {
                nav::detect_type(&entry.path)
            };
            spans.push(Span::styled(type_label, theme::key_hint()));

            Paragraph::new(Line::from(spans))
        }
    };

    f.render_widget(inner_para.block(block), area);
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

fn draw_md_viewer(f: &mut Frame, viewer: &MdViewer, area: Rect, focused: bool) {
    let border_style = if focused {
        Style::default().fg(theme::PHOSPHOR).bg(theme::BG).add_modifier(Modifier::BOLD)
    } else {
        theme::border()
    };
    let title_text = viewer
        .path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| viewer.path.display().to_string());
    let title = Line::from(vec![
        Span::styled(" \u{2592} ", theme::text_dim()),
        Span::styled(title_text, theme::title()),
        Span::styled("  read-only \u{2014} press <e> to edit ", theme::text_dim()),
    ]);

    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(border_style)
        .style(theme::text())
        .title(title);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines = md::render(&viewer.content);
    let para = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((viewer.scroll, 0));
    f.render_widget(para, inner);
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
                } else if app.md_viewer.is_some() {
                    match app.focus {
                        Focus::Left => " [browse]",
                        Focus::Right => " [view]",
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
                    Span::styled("<A-h/l>", theme::key_hint()),
                    Span::styled("resize ", theme::status()),
                    Span::styled("<C-Space>", theme::key_hint()),
                    Span::styled("focus ", theme::status()),
                ];

                if app.md_viewer.is_some() && app.focus == Focus::Right {
                    spans.push(Span::styled("<e>", theme::key_hint()));
                    spans.push(Span::styled("edit ", theme::status()));
                }

                spans.push(Span::styled("<q>", theme::key_hint()));
                spans.push(Span::styled("quit", theme::status()));
                spans.push(Span::styled(
                    format!(
                        "  │ {} dirs, {} files{}{}",
                        dirs, files, hidden_indicator, focus_indicator
                    ),
                    theme::status(),
                ));

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
