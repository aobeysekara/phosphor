use std::path::PathBuf;

use crate::fuzzy;
use crate::input::Action;
use crate::nav::{self, FileEntry};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Search,
}

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
            show_hidden: false,
            status_message: None,
            should_quit: false,
            selected_path: None,
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
                    }
                }
            }

            Action::Select => {
                if let Some(&idx) = self.visible.get(self.cursor) {
                    let entry = &self.entries[idx];
                    if entry.is_dir {
                        self.selected_path = Some(entry.path.clone());
                        self.should_quit = true;
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
                self.recompute_visible();
                // Clamp cursor
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
                // Keep the filter active
            }

            Action::SearchCancel => {
                self.mode = Mode::Normal;
                self.search_query.clear();
                self.recompute_visible();
                self.cursor = 0;
            }

            Action::None => {}
        }
    }

    /// Get the currently selected entry, if any.
    #[allow(dead_code)]
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
