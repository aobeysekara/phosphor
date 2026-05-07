use ratatui::style::{Color, Modifier, Style};

// Near-black with a faint green cast — CRT screen off-state
pub const BG: Color = Color::Rgb(3, 8, 3);

// Slightly lighter for the header bar
pub const HEADER_BG: Color = Color::Rgb(6, 16, 6);

// Standard phosphor glow — P1 green
pub const TEXT_PRIMARY: Color = Color::Rgb(0, 230, 55);

// Dimmed phosphor — further from the beam centre
pub const TEXT_DIM: Color = Color::Rgb(0, 125, 30);

// Peak phosphor glow — used for directories
pub const PHOSPHOR: Color = Color::Rgb(0, 255, 65);

// Dimmed peak glow — alternating directory rows
pub const PHOSPHOR_DIM: Color = Color::Rgb(0, 170, 48);

// Bloom highlight — used for titles and selected directory text
pub const PHOSPHOR_BRIGHT: Color = Color::Rgb(140, 255, 165);

// Vivid orange selection band
pub const SELECTED_BG: Color = Color::Rgb(255, 140, 0);

// Near-black text on the orange selection band, for high contrast
pub const SELECTED_FG: Color = Color::Rgb(15, 8, 0);

// Border — dark phosphor trace
pub const BORDER_COLOUR: Color = Color::Rgb(0, 72, 20);

// Status bar — muted background-level green
pub const STATUS: Color = Color::Rgb(0, 95, 25);

// Error colour
pub const RED: Color = Color::Rgb(255, 60, 40);

/// Default text style.
pub fn text() -> Style {
    Style::default().fg(TEXT_PRIMARY).bg(BG)
}

/// Dimmed text for alternating-row scanline effect.
pub fn text_dim() -> Style {
    Style::default().fg(TEXT_DIM).bg(BG)
}

/// Directory entry style.
pub fn dir_entry() -> Style {
    Style::default().fg(PHOSPHOR).bg(BG)
}

/// Dimmed directory entry for alternating rows.
pub fn dir_entry_dim() -> Style {
    Style::default().fg(PHOSPHOR_DIM).bg(BG)
}

/// Selected file row.
pub fn selected() -> Style {
    Style::default()
        .fg(SELECTED_FG)
        .bg(SELECTED_BG)
        .add_modifier(Modifier::BOLD)
}

/// Selected directory row.
pub fn selected_dir() -> Style {
    Style::default()
        .fg(SELECTED_FG)
        .bg(SELECTED_BG)
        .add_modifier(Modifier::BOLD)
}

/// Border style.
pub fn border() -> Style {
    Style::default().fg(BORDER_COLOUR).bg(BG)
}

/// Header title style.
pub fn title() -> Style {
    Style::default()
        .fg(PHOSPHOR_BRIGHT)
        .bg(HEADER_BG)
        .add_modifier(Modifier::BOLD)
}

/// Header background style.
pub fn header_bg() -> Style {
    Style::default().fg(TEXT_PRIMARY).bg(HEADER_BG)
}

/// Column header labels.
pub fn column_header() -> Style {
    Style::default()
        .fg(TEXT_DIM)
        .bg(BG)
        .add_modifier(Modifier::BOLD)
}

/// Footer / status bar style.
pub fn status() -> Style {
    Style::default().fg(STATUS).bg(BG)
}

/// Key hint accent in footer.
pub fn key_hint() -> Style {
    Style::default().fg(PHOSPHOR).bg(BG)
}

/// Error message style.
pub fn error() -> Style {
    Style::default().fg(RED).bg(BG)
}

/// Search input text style.
pub fn search_input() -> Style {
    Style::default()
        .fg(PHOSPHOR_BRIGHT)
        .bg(BG)
        .add_modifier(Modifier::BOLD)
}
