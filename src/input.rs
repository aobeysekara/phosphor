use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::Mode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    MoveUp,
    MoveDown,
    Open,
    Select,
    GoBack,
    GoHome,
    GoTop,
    GoBottom,
    ToggleHidden,
    EnterSearch,
    // Search mode actions
    SearchInput(char),
    SearchBackspace,
    SearchConfirm,
    SearchCancel,
    None,
}

/// Map a key event to an action based on the current mode.
pub fn handle_key(key: KeyEvent, mode: &Mode) -> Action {
    // Ctrl-C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Action::Quit;
    }

    match mode {
        Mode::Normal => handle_normal(key),
        Mode::Search => handle_search(key),
    }
}

fn handle_normal(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
        KeyCode::Char('l') | KeyCode::Right => Action::Open,
        KeyCode::Enter => Action::Select,
        KeyCode::Backspace | KeyCode::Char('h') | KeyCode::Left => Action::GoBack,
        KeyCode::Char('~') => Action::GoHome,
        KeyCode::Char('g') => Action::GoTop,
        KeyCode::Char('G') => Action::GoBottom,
        KeyCode::Char('.') => Action::ToggleHidden,
        KeyCode::Char('/') => Action::EnterSearch,
        _ => Action::None,
    }
}

fn handle_search(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::SearchCancel,
        KeyCode::Enter => Action::SearchConfirm,
        KeyCode::Backspace => Action::SearchBackspace,
        KeyCode::Up => Action::MoveUp,
        KeyCode::Down => Action::MoveDown,
        KeyCode::Char(c) => Action::SearchInput(c),
        _ => Action::None,
    }
}
