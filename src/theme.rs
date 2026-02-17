use ratatui::style::{Color, Modifier, Style};

// Near-black with warm tint
pub const BG: Color = Color::Rgb(10, 10, 8);

// Classic amber phosphor
pub const AMBER: Color = Color::Rgb(255, 176, 0);

// Dimmed amber for alternating rows / secondary text
pub const AMBER_DIM: Color = Color::Rgb(160, 110, 0);

// P1 green phosphor — directories, search highlights
pub const GREEN: Color = Color::Rgb(51, 255, 0);

// Dimmed green for alternating rows
pub const GREEN_DIM: Color = Color::Rgb(32, 160, 0);

// Muted amber for borders
pub const BORDER: Color = Color::Rgb(80, 56, 0);

// Error / warning colour
pub const RED: Color = Color::Rgb(255, 60, 40);

/// Default text style: amber on near-black.
pub fn text() -> Style {
    Style::default().fg(AMBER).bg(BG)
}

/// Dimmed text for alternating-row scanline effect.
pub fn text_dim() -> Style {
    Style::default().fg(AMBER_DIM).bg(BG)
}

/// Directory entry style: green on near-black.
pub fn dir_entry() -> Style {
    Style::default().fg(GREEN).bg(BG)
}

/// Dimmed directory entry for alternating rows.
pub fn dir_entry_dim() -> Style {
    Style::default().fg(GREEN_DIM).bg(BG)
}

/// Selected row: inverted amber.
pub fn selected() -> Style {
    Style::default().fg(BG).bg(AMBER).add_modifier(Modifier::BOLD)
}

/// Selected directory row: inverted green.
pub fn selected_dir() -> Style {
    Style::default().fg(BG).bg(GREEN).add_modifier(Modifier::BOLD)
}

/// Border style.
pub fn border() -> Style {
    Style::default().fg(BORDER).bg(BG)
}

/// Header title style.
pub fn title() -> Style {
    Style::default()
        .fg(AMBER)
        .bg(BG)
        .add_modifier(Modifier::BOLD)
}

/// Footer / status bar style.
pub fn status() -> Style {
    Style::default().fg(AMBER_DIM).bg(BG)
}

/// Error message style.
pub fn error() -> Style {
    Style::default().fg(RED).bg(BG)
}

/// Search input text style.
pub fn search_input() -> Style {
    Style::default().fg(GREEN).bg(BG).add_modifier(Modifier::BOLD)
}
