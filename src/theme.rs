use ratatui::style::{Color, Modifier, Style};

// Dark navy background (k9s-inspired)
pub const BG: Color = Color::Rgb(13, 17, 23);

// Slightly lighter navy for header bar
pub const HEADER_BG: Color = Color::Rgb(20, 28, 40);

// Primary text — light grey
pub const TEXT_PRIMARY: Color = Color::Rgb(200, 210, 220);

// Dimmed text — muted grey
pub const TEXT_DIM: Color = Color::Rgb(110, 120, 135);

// Directories — bright cyan
pub const CYAN: Color = Color::Rgb(0, 200, 220);

// Dimmed cyan for alternating directory rows
pub const CYAN_DIM: Color = Color::Rgb(0, 140, 155);

// Bright cyan for titles and accents
pub const CYAN_BRIGHT: Color = Color::Rgb(80, 220, 240);

// Teal background for selected row
pub const SELECTED_BG: Color = Color::Rgb(0, 80, 100);

// Steel blue for borders
pub const BORDER_COLOUR: Color = Color::Rgb(58, 80, 120);

// Status bar — muted steel
pub const STATUS: Color = Color::Rgb(90, 110, 140);

// Search input — bold green (kept from original)
pub const GREEN: Color = Color::Rgb(80, 240, 120);

// Error / warning colour
pub const RED: Color = Color::Rgb(255, 60, 40);

/// Default text style: light grey on dark navy.
pub fn text() -> Style {
    Style::default().fg(TEXT_PRIMARY).bg(BG)
}

/// Dimmed text for alternating-row scanline effect.
pub fn text_dim() -> Style {
    Style::default().fg(TEXT_DIM).bg(BG)
}

/// Directory entry style: cyan on dark navy.
pub fn dir_entry() -> Style {
    Style::default().fg(CYAN).bg(BG)
}

/// Dimmed directory entry for alternating rows.
pub fn dir_entry_dim() -> Style {
    Style::default().fg(CYAN_DIM).bg(BG)
}

/// Selected row: bright text on teal background.
pub fn selected() -> Style {
    Style::default()
        .fg(TEXT_PRIMARY)
        .bg(SELECTED_BG)
        .add_modifier(Modifier::BOLD)
}

/// Selected directory row: bright cyan on teal background.
pub fn selected_dir() -> Style {
    Style::default()
        .fg(CYAN_BRIGHT)
        .bg(SELECTED_BG)
        .add_modifier(Modifier::BOLD)
}

/// Border style.
pub fn border() -> Style {
    Style::default().fg(BORDER_COLOUR).bg(BG)
}

/// Header title style: bold bright cyan.
pub fn title() -> Style {
    Style::default()
        .fg(CYAN_BRIGHT)
        .bg(HEADER_BG)
        .add_modifier(Modifier::BOLD)
}

/// Header background style.
pub fn header_bg() -> Style {
    Style::default().fg(TEXT_PRIMARY).bg(HEADER_BG)
}

/// Column header labels (Name, Size).
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

/// Key hint accent (the `<key>` portion in footer).
pub fn key_hint() -> Style {
    Style::default().fg(CYAN).bg(BG)
}

/// Error message style.
pub fn error() -> Style {
    Style::default().fg(RED).bg(BG)
}

/// Search input text style.
pub fn search_input() -> Style {
    Style::default().fg(GREEN).bg(BG).add_modifier(Modifier::BOLD)
}
