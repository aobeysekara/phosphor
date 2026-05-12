use std::fs;
use std::path::{Path, PathBuf};
use std::str::Chars;

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::theme;

pub struct MdViewer {
    pub path: PathBuf,
    pub content: String,
    pub scroll: u16,
}

impl MdViewer {
    /// Load a markdown file from disk into memory.
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("read {}: {}", path.display(), e))?;
        Ok(MdViewer { path: path.to_path_buf(), content, scroll: 0 })
    }

    pub fn line_count(&self) -> usize {
        self.content.lines().count().max(1)
    }

    pub fn scroll_down(&mut self, n: u16) {
        let max = self.line_count().saturating_sub(1).min(u16::MAX as usize) as u16;
        self.scroll = self.scroll.saturating_add(n).min(max);
    }

    pub fn scroll_up(&mut self, n: u16) {
        self.scroll = self.scroll.saturating_sub(n);
    }

    pub fn scroll_top(&mut self) {
        self.scroll = 0;
    }

    pub fn scroll_bottom(&mut self) {
        self.scroll = self.line_count().saturating_sub(1).min(u16::MAX as usize) as u16;
    }
}

/// True if the path's extension marks it as markdown.
pub fn is_markdown(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).as_deref(),
        Some("md") | Some("markdown") | Some("mdown") | Some("mkd")
    )
}

/// Render markdown content into styled ratatui lines.
pub fn render(content: &str) -> Vec<Line<'static>> {
    let mut out: Vec<Line<'static>> = Vec::new();
    let mut in_code = false;
    for raw in content.lines() {
        let line = raw.trim_end_matches('\r');
        let trimmed = line.trim_start();

        if let Some(rest) = trimmed.strip_prefix("```") {
            if in_code {
                in_code = false;
                out.push(Line::from(Span::styled(fence_glyph(""), theme::text_dim())));
            } else {
                in_code = true;
                let lang = rest.trim();
                out.push(Line::from(Span::styled(fence_glyph(lang), theme::text_dim())));
            }
            continue;
        }

        if in_code {
            out.push(Line::from(Span::styled(line.to_string(), code_block_style())));
            continue;
        }

        out.push(render_line(line));
    }
    out
}

fn fence_glyph(lang: &str) -> String {
    if lang.is_empty() {
        "─── code".to_string()
    } else {
        format!("─── {}", lang)
    }
}

fn render_line(line: &str) -> Line<'static> {
    let leading_len = line.chars().take_while(|c| c.is_whitespace()).count();
    let leading: String = line.chars().take(leading_len).collect();
    let body: String = line.chars().skip(leading_len).collect();

    if body.is_empty() {
        return Line::from(Span::raw(""));
    }

    let mut hashes = 0;
    for c in body.chars() {
        if c == '#' && hashes < 6 {
            hashes += 1;
        } else {
            break;
        }
    }
    if hashes > 0 && body.chars().nth(hashes) == Some(' ') {
        let rest: String = body.chars().skip(hashes + 1).collect();
        let prefix = "#".repeat(hashes);
        let mut spans: Vec<Span<'static>> = Vec::new();
        if !leading.is_empty() {
            spans.push(Span::raw(leading));
        }
        spans.push(Span::styled(
            format!("{} {}", prefix, rest),
            heading_style(hashes),
        ));
        return Line::from(spans);
    }

    if body.starts_with("- ") || body.starts_with("* ") || body.starts_with("+ ") {
        let item: String = body.chars().skip(2).collect();
        let mut spans: Vec<Span<'static>> = Vec::new();
        if !leading.is_empty() {
            spans.push(Span::raw(leading));
        }
        spans.push(Span::styled("• ".to_string(), bullet_style()));
        spans.extend(render_inline(&item, theme::text()));
        return Line::from(spans);
    }

    if let Some(after) = strip_numbered_prefix(&body) {
        let prefix_len = body.len() - after.len();
        let prefix: String = body.chars().take(prefix_len).collect();
        let mut spans: Vec<Span<'static>> = Vec::new();
        if !leading.is_empty() {
            spans.push(Span::raw(leading));
        }
        spans.push(Span::styled(prefix, bullet_style()));
        spans.extend(render_inline(&after, theme::text()));
        return Line::from(spans);
    }

    if let Some(rest) = body.strip_prefix("> ") {
        let mut spans: Vec<Span<'static>> = Vec::new();
        if !leading.is_empty() {
            spans.push(Span::raw(leading));
        }
        spans.push(Span::styled("│ ".to_string(), quote_bar_style()));
        spans.extend(render_inline(rest, theme::text_dim()));
        return Line::from(spans);
    }

    if body == "---" || body == "***" || body == "___" {
        return Line::from(Span::styled("─".repeat(40), theme::text_dim()));
    }

    let mut spans: Vec<Span<'static>> = if leading.is_empty() {
        Vec::new()
    } else {
        vec![Span::raw(leading)]
    };
    spans.extend(render_inline(&body, theme::text()));
    Line::from(spans)
}

fn strip_numbered_prefix(body: &str) -> Option<String> {
    let dot = body.find('.')?;
    let num = &body[..dot];
    if num.is_empty() || !num.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let after = &body[dot + 1..];
    let after = after.strip_prefix(' ')?;
    Some(after.to_string())
}

fn render_inline(text: &str, base: Style) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '`' {
            if !current.is_empty() {
                spans.push(Span::styled(std::mem::take(&mut current), base));
            }
            let (text, closed) = consume_until_char(&mut chars, '`');
            if closed {
                spans.push(Span::styled(text, inline_code_style()));
            } else {
                current.push('`');
                current.push_str(&text);
            }
            continue;
        }
        if c == '*' {
            if chars.peek() == Some(&'*') {
                chars.next();
                if !current.is_empty() {
                    spans.push(Span::styled(std::mem::take(&mut current), base));
                }
                let (text, closed) = consume_until_double_star(&mut chars);
                if closed {
                    spans.push(Span::styled(text, base.add_modifier(Modifier::BOLD)));
                } else {
                    current.push_str("**");
                    current.push_str(&text);
                }
                continue;
            }
            if !current.is_empty() {
                spans.push(Span::styled(std::mem::take(&mut current), base));
            }
            let (text, closed) = consume_until_char(&mut chars, '*');
            if closed {
                spans.push(Span::styled(text, base.add_modifier(Modifier::ITALIC)));
            } else {
                current.push('*');
                current.push_str(&text);
            }
            continue;
        }
        current.push(c);
    }
    if !current.is_empty() {
        spans.push(Span::styled(current, base));
    }
    spans
}

fn consume_until_char(chars: &mut std::iter::Peekable<Chars>, target: char) -> (String, bool) {
    let mut buf = String::new();
    for c in chars.by_ref() {
        if c == target {
            return (buf, true);
        }
        buf.push(c);
    }
    (buf, false)
}

fn consume_until_double_star(chars: &mut std::iter::Peekable<Chars>) -> (String, bool) {
    let mut buf = String::new();
    while let Some(c) = chars.next() {
        if c == '*' && chars.peek() == Some(&'*') {
            chars.next();
            return (buf, true);
        }
        buf.push(c);
    }
    (buf, false)
}

fn heading_style(level: usize) -> Style {
    match level {
        1 => Style::default()
            .fg(theme::ORANGE)
            .bg(theme::BG)
            .add_modifier(Modifier::BOLD),
        2 => Style::default()
            .fg(theme::PHOSPHOR_BRIGHT)
            .bg(theme::BG)
            .add_modifier(Modifier::BOLD),
        3 => Style::default()
            .fg(theme::PHOSPHOR)
            .bg(theme::BG)
            .add_modifier(Modifier::BOLD),
        _ => Style::default()
            .fg(theme::TEXT_PRIMARY)
            .bg(theme::BG)
            .add_modifier(Modifier::BOLD),
    }
}

fn bullet_style() -> Style {
    Style::default().fg(theme::PHOSPHOR).bg(theme::BG)
}

fn quote_bar_style() -> Style {
    Style::default().fg(theme::ORANGE).bg(theme::BG)
}

fn code_block_style() -> Style {
    Style::default().fg(theme::PHOSPHOR_BRIGHT).bg(theme::HEADER_BG)
}

fn inline_code_style() -> Style {
    Style::default()
        .fg(theme::PHOSPHOR_BRIGHT)
        .bg(theme::HEADER_BG)
        .add_modifier(Modifier::BOLD)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_md_extension() {
        assert!(is_markdown(Path::new("foo.md")));
        assert!(is_markdown(Path::new("foo.MD")));
        assert!(is_markdown(Path::new("foo.markdown")));
        assert!(!is_markdown(Path::new("foo.txt")));
        assert!(!is_markdown(Path::new("foo")));
    }

    #[test]
    fn parses_heading_as_bold() {
        let out = render("# Title");
        assert_eq!(out.len(), 1);
        let has_bold = out[0]
            .spans
            .iter()
            .any(|s| s.style.add_modifier.contains(Modifier::BOLD));
        assert!(has_bold);
    }

    #[test]
    fn parses_inline_code_as_styled_span() {
        let out = render("hello `code` world");
        let line = &out[0];
        let texts: Vec<String> = line.spans.iter().map(|s| s.content.to_string()).collect();
        assert!(texts.iter().any(|t| t == "code"));
    }

    #[test]
    fn parses_bullet_with_glyph() {
        let out = render("- item");
        let line = &out[0];
        let texts: Vec<String> = line.spans.iter().map(|s| s.content.to_string()).collect();
        assert!(texts.iter().any(|t| t.contains('•')));
        assert!(texts.iter().any(|t| t.contains("item")));
    }

    #[test]
    fn numbered_list_keeps_prefix() {
        let out = render("1. first");
        let line = &out[0];
        let texts: Vec<String> = line.spans.iter().map(|s| s.content.to_string()).collect();
        assert!(texts.iter().any(|t| t.contains("1.")));
        assert!(texts.iter().any(|t| t.contains("first")));
    }

    #[test]
    fn fenced_code_block_emits_marker_lines() {
        let out = render("```rust\nfn main() {}\n```");
        assert_eq!(out.len(), 3);
    }

    #[test]
    fn unclosed_inline_code_is_kept_literal() {
        let out = render("hello `world");
        let combined: String = out[0]
            .spans
            .iter()
            .map(|s| s.content.to_string())
            .collect::<Vec<_>>()
            .join("");
        assert!(combined.contains("`world"));
    }

    #[test]
    fn bold_marker_produces_bold_span() {
        let out = render("text **bold** end");
        let line = &out[0];
        let bold_span = line
            .spans
            .iter()
            .find(|s| s.style.add_modifier.contains(Modifier::BOLD));
        assert!(bold_span.is_some());
        assert_eq!(bold_span.unwrap().content.to_string(), "bold");
    }

    #[test]
    fn scroll_clamps_to_line_count() {
        let mut viewer = MdViewer {
            path: PathBuf::from("/dev/null"),
            content: "a\nb\nc".to_string(),
            scroll: 0,
        };
        viewer.scroll_down(100);
        assert_eq!(viewer.scroll, 2);
        viewer.scroll_up(5);
        assert_eq!(viewer.scroll, 0);
        viewer.scroll_bottom();
        assert_eq!(viewer.scroll, 2);
        viewer.scroll_top();
        assert_eq!(viewer.scroll, 0);
    }
}
