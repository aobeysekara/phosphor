mod app;
mod editor;
mod fuzzy;
mod input;
mod md;
mod nav;
mod theme;
mod tree;
mod ui;

use std::env;
use std::io;
use std::time::Duration;

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton,
    MouseEventKind,
};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::Terminal;

use app::{App, Focus};
use editor::{key_to_bytes, Editor};
use input::{handle_key, handle_md_viewer_key, resize_action};
use ui::{main_area, right_panel_size};

fn main() -> io::Result<()> {
    let start_dir = env::args()
        .nth(1)
        .map(Into::into)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| ".".into()));

    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let selected = run(&mut terminal, start_dir);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), DisableMouseCapture, LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Some(path) = selected? {
        println!("{}", path.display());
    }

    Ok(())
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stderr>>,
    start_dir: std::path::PathBuf,
) -> io::Result<Option<std::path::PathBuf>> {
    let mut app = App::new(start_dir);
    let mut editor: Option<Editor> = None;

    loop {
        // Detect editor exit and clean up.
        if let Some(ed) = editor.as_mut() {
            if !ed.is_alive() {
                editor = None;
                app.editor_alive = false;
                app.focus = Focus::Left;
            }
        }

        // Keep the pty in sync with the right panel size.
        let term_area = terminal.size()?;
        let term_rect = ratatui::layout::Rect::new(0, 0, term_area.width, term_area.height);
        let (panel_cols, panel_rows) = right_panel_size(term_rect, app.right_pct);
        if let Some(ed) = editor.as_mut() {
            let _ = ed.resize(panel_rows.max(1), panel_cols.max(1));
        }

        // Service any pending "open file" request from the left panel or the
        // markdown viewer's edit-switch.
        if let Some(path) = app.open_in_editor.take() {
            // Editor and viewer are mutually exclusive in the right panel.
            app.md_viewer = None;
            match editor.as_mut() {
                Some(ed) => {
                    if let Err(e) = ed.open_file(&path) {
                        app.status_message = Some(format!("vim: {}", e));
                    }
                }
                None => match Editor::spawn(panel_rows.max(1), panel_cols.max(1), &path) {
                    Ok(ed) => {
                        editor = Some(ed);
                        app.editor_alive = true;
                    }
                    Err(e) => app.status_message = Some(format!("vim: {}", e)),
                },
            }
            if app.editor_alive {
                app.focus = Focus::Right;
            }
        }

        // If the viewer was just opened, ensure any prior editor session is
        // closed — the right panel can only host one widget at a time.
        if app.md_viewer.is_some() && editor.is_some() {
            editor = None;
            app.editor_alive = false;
            app.focus = Focus::Right;
        }

        terminal.draw(|f| ui::draw(f, &app, editor.as_ref()))?;

        if event::poll(Duration::from_millis(50))? {
            handle_event(event::read()?, &mut app, editor.as_mut(), term_rect);
            while event::poll(Duration::from_millis(0))? {
                handle_event(event::read()?, &mut app, editor.as_mut(), term_rect);
            }
        }

        if app.should_quit {
            return Ok(app.selected_path);
        }
    }
}

fn handle_event(
    event: Event,
    app: &mut App,
    editor: Option<&mut Editor>,
    term_rect: ratatui::layout::Rect,
) {
    match event {
        Event::Key(key) => handle_key_event(key, app, editor),
        Event::Mouse(mouse) => {
            let area = main_area(term_rect);
            match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    app.start_drag(mouse.column, mouse.row, area);
                }
                MouseEventKind::Drag(MouseButton::Left) => {
                    app.update_drag(mouse.column, area);
                }
                MouseEventKind::Up(MouseButton::Left) => {
                    app.end_drag();
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn handle_key_event(
    key: ratatui::crossterm::event::KeyEvent,
    app: &mut App,
    editor: Option<&mut Editor>,
) {
    let is_focus_toggle =
        key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char(' ');

    if is_focus_toggle {
        let right_active = app.has_right_panel();
        app.focus = match (app.focus, right_active) {
            (Focus::Right, _) => Focus::Left,
            (Focus::Left, true) => Focus::Right,
            (Focus::Left, false) => Focus::Left,
        };
        return;
    }

    if let Some(action) = resize_action(key) {
        app.update(action);
        return;
    }

    if app.focus == Focus::Right {
        if app.md_viewer.is_some() {
            let action = handle_md_viewer_key(key);
            app.update(action);
        } else if let Some(ed) = editor {
            let bytes = key_to_bytes(key);
            if !bytes.is_empty() {
                let _ = ed.send_bytes(&bytes);
            }
        }
    } else {
        let action = handle_key(key, &app.mode);
        app.update(action);
    }
}
