use std::path::PathBuf;

use ratatui::layout::Rect;

use crate::fuzzy;
use crate::input::Action;
use crate::md::{self, MdViewer};
use crate::nav::{self, FileEntry};
use crate::tree::{self, TreeNode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Splitter {
    TreeList,
    ListRight,
}

pub const TREE_PCT_MIN: u16 = 10;
pub const TREE_PCT_MAX: u16 = 50;
pub const RIGHT_PCT_MIN: u16 = 20;
pub const RIGHT_PCT_MAX: u16 = 75;
pub const LIST_PCT_MIN: u16 = 15;
pub const RESIZE_STEP: u16 = 5;

pub struct App {
    pub current_dir: PathBuf,
    pub entries: Vec<FileEntry>,
    /// Indices into `entries` that are currently visible (after hidden filter + fuzzy search).
    pub visible: Vec<usize>,
    /// Cursor position within `visible`.
    pub cursor: usize,
    pub mode: Mode,
    pub search_query: String,
    pub show_hidden: bool,
    pub status_message: Option<String>,
    pub should_quit: bool,
    pub selected_path: Option<PathBuf>,
    pub open_in_editor: Option<PathBuf>,
    pub focus: Focus,
    pub editor_alive: bool,
    pub tree: Vec<TreeNode>,
    pub md_viewer: Option<MdViewer>,
    pub tree_pct: u16,
    pub right_pct: u16,
    pub drag: Option<Splitter>,
    pub mouse_capture: bool,
}

impl App {
    pub fn new(start_dir: PathBuf) -> Self {
        let mut app = App {
            current_dir: start_dir,
            entries: Vec::new(),
            visible: Vec::new(),
            cursor: 0,
            mode: Mode::Normal,
            search_query: String::new(),
            show_hidden: true,
            status_message: None,
            should_quit: false,
            selected_path: None,
            open_in_editor: None,
            focus: Focus::Left,
            editor_alive: false,
            tree: Vec::new(),
            md_viewer: None,
            tree_pct: 25,
            right_pct: 40,
            drag: None,
            mouse_capture: true,
        };
        app.load_directory();
        app
    }

    /// Load entries from the current directory.
    pub fn load_directory(&mut self) {
        match nav::read_directory(&self.current_dir) {
            Ok(entries) => {
                self.entries = entries;
                self.status_message = None;
            }
            Err(msg) => {
                self.entries = Vec::new();
                self.status_message = Some(msg);
            }
        }
        self.tree = tree::build(&self.current_dir, self.show_hidden);
        self.recompute_visible();
        self.cursor = 0;
    }

    /// Recompute the visible indices based on hidden filter and search query.
    fn recompute_visible(&mut self) {
        // First, filter by hidden
        let filtered: Vec<FileEntry> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| self.show_hidden || !e.is_hidden)
            .map(|(_, e)| e.clone())
            .collect();

        // Build a temporary vec for fuzzy matching, keeping track of original indices
        let non_hidden_indices: Vec<usize> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| self.show_hidden || !e.is_hidden)
            .map(|(i, _)| i)
            .collect();

        if self.search_query.is_empty() {
            self.visible = non_hidden_indices;
        } else {
            // Fuzzy filter against the non-hidden entries
            let fuzzy_indices = fuzzy::filter_entries(&filtered, &self.search_query);
            self.visible = fuzzy_indices
                .into_iter()
                .map(|fi| non_hidden_indices[fi])
                .collect();
        }
    }

    /// True if there is something to show in the right panel.
    pub fn has_right_panel(&self) -> bool {
        self.editor_alive || self.md_viewer.is_some()
    }

    /// Open the selected file: markdown goes to the viewer, everything else to vim.
    fn open_file_path(&mut self, path: PathBuf) {
        if md::is_markdown(&path) {
            match MdViewer::load(&path) {
                Ok(v) => {
                    self.md_viewer = Some(v);
                    self.status_message = None;
                }
                Err(e) => self.status_message = Some(format!("md: {}", e)),
            }
        } else {
            self.open_in_editor = Some(path);
        }
    }

    /// Process an action and update state.
    pub fn update(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,

            Action::MoveUp => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }

            Action::MoveDown => {
                if !self.visible.is_empty() && self.cursor < self.visible.len() - 1 {
                    self.cursor += 1;
                }
            }

            Action::Open => {
                if let Some(&idx) = self.visible.get(self.cursor) {
                    let entry = &self.entries[idx];
                    if entry.is_dir {
                        let new_dir = entry.path.clone();
                        self.current_dir = new_dir;
                        self.search_query.clear();
                        self.mode = Mode::Normal;
                        self.load_directory();
                    } else {
                        let path = entry.path.clone();
                        self.open_file_path(path);
                    }
                }
            }

            Action::Select => {
                if let Some(&idx) = self.visible.get(self.cursor) {
                    let entry = &self.entries[idx];
                    if entry.is_dir {
                        self.selected_path = Some(entry.path.clone());
                        self.should_quit = true;
                    } else {
                        let path = entry.path.clone();
                        self.open_file_path(path);
                    }
                }
            }

            Action::GoBack => {
                if let Some(parent) = self.current_dir.parent() {
                    let parent = parent.to_path_buf();
                    self.current_dir = parent;
                    self.search_query.clear();
                    self.mode = Mode::Normal;
                    self.load_directory();
                }
            }

            Action::GoHome => {
                if let Some(home) = dirs::home_dir() {
                    self.current_dir = home;
                    self.search_query.clear();
                    self.mode = Mode::Normal;
                    self.load_directory();
                }
            }

            Action::GoTop => {
                self.cursor = 0;
            }

            Action::GoBottom => {
                if !self.visible.is_empty() {
                    self.cursor = self.visible.len() - 1;
                }
            }

            Action::ToggleHidden => {
                self.show_hidden = !self.show_hidden;
                self.tree = tree::build(&self.current_dir, self.show_hidden);
                self.recompute_visible();
                if !self.visible.is_empty() {
                    self.cursor = self.cursor.min(self.visible.len() - 1);
                } else {
                    self.cursor = 0;
                }
            }

            Action::EnterSearch => {
                self.mode = Mode::Search;
                self.search_query.clear();
                self.recompute_visible();
                self.cursor = 0;
            }

            Action::SearchInput(c) => {
                self.search_query.push(c);
                self.recompute_visible();
                self.cursor = 0;
            }

            Action::SearchBackspace => {
                self.search_query.pop();
                self.recompute_visible();
                self.cursor = 0;
            }

            Action::SearchConfirm => {
                self.mode = Mode::Normal;
            }

            Action::SearchCancel => {
                self.mode = Mode::Normal;
                self.search_query.clear();
                self.recompute_visible();
                self.cursor = 0;
            }

            Action::MdScrollDown => {
                if let Some(v) = self.md_viewer.as_mut() {
                    v.scroll_down(1);
                }
            }

            Action::MdScrollUp => {
                if let Some(v) = self.md_viewer.as_mut() {
                    v.scroll_up(1);
                }
            }

            Action::MdPageDown => {
                if let Some(v) = self.md_viewer.as_mut() {
                    v.scroll_down(10);
                }
            }

            Action::MdPageUp => {
                if let Some(v) = self.md_viewer.as_mut() {
                    v.scroll_up(10);
                }
            }

            Action::MdTop => {
                if let Some(v) = self.md_viewer.as_mut() {
                    v.scroll_top();
                }
            }

            Action::MdBottom => {
                if let Some(v) = self.md_viewer.as_mut() {
                    v.scroll_bottom();
                }
            }

            Action::MdEdit => {
                if let Some(v) = self.md_viewer.as_ref() {
                    self.open_in_editor = Some(v.path.clone());
                }
            }

            Action::CloseRight => {
                self.md_viewer = None;
                if self.focus == Focus::Right {
                    self.focus = Focus::Left;
                }
            }

            Action::ResizeShrink => self.resize_focused(-(RESIZE_STEP as i16)),
            Action::ResizeGrow => self.resize_focused(RESIZE_STEP as i16),

            Action::ToggleMouseCapture => {
                self.mouse_capture = !self.mouse_capture;
                self.drag = None;
            }

            Action::None => {}
        }
    }

    /// Apply `delta` percent to the panel whose size belongs to the current focus.
    /// Constraints keep each panel within reasonable bounds and the list panel
    /// from collapsing below `LIST_PCT_MIN`.
    fn resize_focused(&mut self, delta: i16) {
        let right_active = self.has_right_panel();
        match self.focus {
            Focus::Left => {
                let new = clamp_pct(self.tree_pct, delta, TREE_PCT_MIN, TREE_PCT_MAX);
                let right = if right_active { self.right_pct } else { 0 };
                if 100u16.saturating_sub(new).saturating_sub(right) >= LIST_PCT_MIN {
                    self.tree_pct = new;
                }
            }
            Focus::Right => {
                if !right_active {
                    return;
                }
                let new = clamp_pct(self.right_pct, delta, RIGHT_PCT_MIN, RIGHT_PCT_MAX);
                if 100u16.saturating_sub(self.tree_pct).saturating_sub(new) >= LIST_PCT_MIN {
                    self.right_pct = new;
                }
            }
        }
    }

    /// Begin a panel-resize drag if `(x, y)` lands on a splitter column.
    /// Returns true if a drag was started.
    pub fn start_drag(&mut self, x: u16, y: u16, main_area: Rect) -> bool {
        if !point_in_area(x, y, main_area) || main_area.width == 0 {
            return false;
        }
        let tree_boundary = boundary_col(main_area, self.tree_pct);
        if near_column(x, tree_boundary) {
            self.drag = Some(Splitter::TreeList);
            return true;
        }
        if self.has_right_panel() {
            let right_width =
                (main_area.width as u32 * self.right_pct as u32 / 100) as u16;
            let right_boundary = main_area.x + main_area.width.saturating_sub(right_width);
            if near_column(x, right_boundary) {
                self.drag = Some(Splitter::ListRight);
                return true;
            }
        }
        false
    }

    /// Update the currently dragged splitter to follow `x`. No-op if no drag is active.
    pub fn update_drag(&mut self, x: u16, main_area: Rect) {
        let Some(splitter) = self.drag else { return };
        if main_area.width == 0 {
            return;
        }
        let rel_x = x
            .saturating_sub(main_area.x)
            .min(main_area.width.saturating_sub(1));
        let width = main_area.width as u32;
        match splitter {
            Splitter::TreeList => {
                let raw = (rel_x as u32 * 100 / width) as u16;
                let new = raw.clamp(TREE_PCT_MIN, TREE_PCT_MAX);
                let right = if self.has_right_panel() { self.right_pct } else { 0 };
                let max_allowed = 100u16
                    .saturating_sub(right)
                    .saturating_sub(LIST_PCT_MIN);
                self.tree_pct = new.min(max_allowed);
            }
            Splitter::ListRight => {
                if !self.has_right_panel() {
                    self.drag = None;
                    return;
                }
                let right_width = main_area.width.saturating_sub(rel_x) as u32;
                let raw = (right_width * 100 / width) as u16;
                let new = raw.clamp(RIGHT_PCT_MIN, RIGHT_PCT_MAX);
                let max_allowed = 100u16
                    .saturating_sub(self.tree_pct)
                    .saturating_sub(LIST_PCT_MIN);
                self.right_pct = new.min(max_allowed);
            }
        }
    }

    /// End any in-progress drag.
    pub fn end_drag(&mut self) {
        self.drag = None;
    }

    /// Get the currently selected entry, if any.
    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.visible
            .get(self.cursor)
            .map(|&idx| &self.entries[idx])
    }

    /// Count of visible directories.
    pub fn dir_count(&self) -> usize {
        self.visible
            .iter()
            .filter(|&&i| self.entries[i].is_dir)
            .count()
    }

    /// Count of visible files.
    pub fn file_count(&self) -> usize {
        self.visible
            .iter()
            .filter(|&&i| !self.entries[i].is_dir)
            .count()
    }

    /// Format the current directory path, replacing home dir with ~.
    pub fn display_path(&self) -> String {
        let path = self.current_dir.display().to_string();
        if let Some(home) = dirs::home_dir() {
            let home_str = home.display().to_string();
            if path == home_str {
                return "~".to_string();
            }
            if let Some(rest) = path.strip_prefix(&home_str) {
                return format!("~{}", rest);
            }
        }
        path
    }
}

fn clamp_pct(current: u16, delta: i16, min: u16, max: u16) -> u16 {
    let signed = current as i32 + delta as i32;
    signed.clamp(min as i32, max as i32) as u16
}

fn boundary_col(area: Rect, pct: u16) -> u16 {
    area.x + (area.width as u32 * pct as u32 / 100) as u16
}

fn point_in_area(x: u16, y: u16, area: Rect) -> bool {
    x >= area.x
        && x < area.x.saturating_add(area.width)
        && y >= area.y
        && y < area.y.saturating_add(area.height)
}

fn near_column(x: u16, col: u16) -> bool {
    let lo = col.saturating_sub(1);
    let hi = col.saturating_add(1);
    x >= lo && x <= hi
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        App {
            current_dir: PathBuf::from("/"),
            entries: Vec::new(),
            visible: Vec::new(),
            cursor: 0,
            mode: Mode::Normal,
            search_query: String::new(),
            show_hidden: true,
            status_message: None,
            should_quit: false,
            selected_path: None,
            open_in_editor: None,
            focus: Focus::Left,
            editor_alive: false,
            tree: Vec::new(),
            md_viewer: None,
            tree_pct: 25,
            right_pct: 40,
            drag: None,
            mouse_capture: true,
        }
    }

    #[test]
    fn resize_left_focus_grows_tree() {
        let mut a = app();
        a.update(Action::ResizeGrow);
        assert_eq!(a.tree_pct, 30);
    }

    #[test]
    fn resize_left_focus_shrinks_tree_to_minimum() {
        let mut a = app();
        for _ in 0..20 {
            a.update(Action::ResizeShrink);
        }
        assert_eq!(a.tree_pct, TREE_PCT_MIN);
    }

    #[test]
    fn resize_right_focus_requires_active_right_panel() {
        let mut a = app();
        a.focus = Focus::Right;
        a.update(Action::ResizeGrow);
        assert_eq!(a.right_pct, 40, "no right panel, right_pct must not change");
    }

    #[test]
    fn resize_right_focus_grows_right_when_active() {
        let mut a = app();
        a.editor_alive = true;
        a.focus = Focus::Right;
        a.update(Action::ResizeGrow);
        assert_eq!(a.right_pct, 45);
    }

    #[test]
    fn resize_right_panel_caps_to_keep_list_visible() {
        let mut a = app();
        a.editor_alive = true;
        a.focus = Focus::Right;
        for _ in 0..20 {
            a.update(Action::ResizeGrow);
        }
        assert!(a.right_pct <= RIGHT_PCT_MAX);
        assert!(100 - a.tree_pct - a.right_pct >= LIST_PCT_MIN);
    }

    fn main_area() -> Rect {
        Rect::new(0, 6, 100, 20)
    }

    #[test]
    fn start_drag_on_tree_boundary_begins_tree_list_drag() {
        let mut a = app();
        assert!(a.start_drag(25, 10, main_area()));
        assert_eq!(a.drag, Some(Splitter::TreeList));
    }

    #[test]
    fn start_drag_on_right_boundary_begins_list_right_drag() {
        let mut a = app();
        a.editor_alive = true;
        assert!(a.start_drag(60, 10, main_area()));
        assert_eq!(a.drag, Some(Splitter::ListRight));
    }

    #[test]
    fn start_drag_in_panel_interior_does_nothing() {
        let mut a = app();
        a.editor_alive = true;
        assert!(!a.start_drag(45, 10, main_area()));
        assert!(a.drag.is_none());
    }

    #[test]
    fn start_drag_outside_main_area_is_ignored() {
        let mut a = app();
        assert!(!a.start_drag(25, 2, main_area()));
    }

    #[test]
    fn start_drag_on_right_boundary_without_right_panel_is_ignored() {
        let mut a = app();
        // right panel inactive, so the only boundary lives at tree_pct=25
        assert!(!a.start_drag(60, 10, main_area()));
    }

    #[test]
    fn update_drag_moves_tree_splitter() {
        let mut a = app();
        a.start_drag(25, 10, main_area());
        a.update_drag(40, main_area());
        assert_eq!(a.tree_pct, 40);
    }

    #[test]
    fn update_drag_moves_right_splitter() {
        let mut a = app();
        a.editor_alive = true;
        a.start_drag(60, 10, main_area());
        a.update_drag(50, main_area());
        assert_eq!(a.right_pct, 50);
    }

    #[test]
    fn update_drag_clamps_tree_to_minimum() {
        let mut a = app();
        a.start_drag(25, 10, main_area());
        a.update_drag(2, main_area());
        assert_eq!(a.tree_pct, TREE_PCT_MIN);
    }

    #[test]
    fn update_drag_clamps_to_keep_list_visible() {
        let mut a = app();
        a.editor_alive = true;
        a.start_drag(60, 10, main_area());
        a.update_drag(30, main_area()); // would give right_pct=70, must respect list min
        assert!(100 - a.tree_pct - a.right_pct >= LIST_PCT_MIN);
        assert!(a.right_pct <= RIGHT_PCT_MAX);
    }

    #[test]
    fn end_drag_clears_state() {
        let mut a = app();
        a.start_drag(25, 10, main_area());
        a.end_drag();
        assert!(a.drag.is_none());
    }

    #[test]
    fn update_drag_without_active_drag_is_noop() {
        let mut a = app();
        a.update_drag(80, main_area());
        assert_eq!(a.tree_pct, 25);
        assert_eq!(a.right_pct, 40);
    }

    #[test]
    fn toggle_mouse_capture_flips_and_clears_drag() {
        let mut a = app();
        a.start_drag(25, 10, main_area());
        assert!(a.drag.is_some());
        a.update(Action::ToggleMouseCapture);
        assert!(!a.mouse_capture);
        assert!(a.drag.is_none());
        a.update(Action::ToggleMouseCapture);
        assert!(a.mouse_capture);
    }
}
