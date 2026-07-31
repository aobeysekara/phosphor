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
    // Markdown viewer actions
    MdScrollDown,
    MdScrollUp,
    MdPageDown,
    MdPageUp,
    MdTop,
    MdBottom,
    MdEdit,
    CloseRight,
    // Layout actions
    ResizeShrink,
    ResizeGrow,
    ToggleMouseCapture,
    None,
}

/// Map a key event to an action for the file browser.
pub fn handle_key(key: KeyEvent, mode: &Mode) -> Action {
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

/// Map a key event to an action while the markdown viewer is focused.
pub fn handle_md_viewer_key(key: KeyEvent) -> Action {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Action::Quit;
    }
    match key.code {
        KeyCode::Char('q') => Action::CloseRight,
        KeyCode::Esc => Action::CloseRight,
        KeyCode::Char('e') => Action::MdEdit,
        KeyCode::Char('j') | KeyCode::Down => Action::MdScrollDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MdScrollUp,
        KeyCode::Char('d') | KeyCode::PageDown => Action::MdPageDown,
        KeyCode::Char('u') | KeyCode::PageUp => Action::MdPageUp,
        KeyCode::Char(' ') => Action::MdPageDown,
        KeyCode::Char('g') | KeyCode::Home => Action::MdTop,
        KeyCode::Char('G') | KeyCode::End => Action::MdBottom,
        _ => Action::None,
    }
}

/// Detect a global keybinding (Alt-modified) regardless of focus. Currently
/// covers panel resize (`Alt+H/L`) and the mouse-capture toggle (`Alt+M`).
pub fn global_action(key: KeyEvent) -> Option<Action> {
    if !key.modifiers.contains(KeyModifiers::ALT) {
        return None;
    }
    match key.code {
        KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Left => Some(Action::ResizeShrink),
        KeyCode::Char('l') | KeyCode::Char('L') | KeyCode::Right => Some(Action::ResizeGrow),
        KeyCode::Char('m') | KeyCode::Char('M') => Some(Action::ToggleMouseCapture),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(code: KeyCode, mods: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, mods)
    }

    #[test]
    fn alt_h_is_resize_shrink() {
        assert_eq!(
            global_action(k(KeyCode::Char('h'), KeyModifiers::ALT)),
            Some(Action::ResizeShrink),
        );
    }

    #[test]
    fn alt_l_is_resize_grow() {
        assert_eq!(
            global_action(k(KeyCode::Char('l'), KeyModifiers::ALT)),
            Some(Action::ResizeGrow),
        );
    }

    #[test]
    fn alt_m_toggles_mouse_capture() {
        assert_eq!(
            global_action(k(KeyCode::Char('m'), KeyModifiers::ALT)),
            Some(Action::ToggleMouseCapture),
        );
    }

    #[test]
    fn plain_h_is_not_global() {
        assert!(global_action(k(KeyCode::Char('h'), KeyModifiers::NONE)).is_none());
    }

    #[test]
    fn viewer_e_switches_to_edit() {
        assert_eq!(
            handle_md_viewer_key(k(KeyCode::Char('e'), KeyModifiers::NONE)),
            Action::MdEdit,
        );
    }

    #[test]
    fn viewer_j_scrolls_down() {
        assert_eq!(
            handle_md_viewer_key(k(KeyCode::Char('j'), KeyModifiers::NONE)),
            Action::MdScrollDown,
        );
    }

    #[test]
    fn viewer_q_closes_right_panel() {
        assert_eq!(
            handle_md_viewer_key(k(KeyCode::Char('q'), KeyModifiers::NONE)),
            Action::CloseRight,
        );
    }
}
