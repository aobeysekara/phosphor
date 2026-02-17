mod app;
mod fuzzy;
mod input;
mod nav;
mod theme;
mod ui;

use std::env;
use std::io;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::App;
use input::handle_key;

fn main() -> io::Result<()> {
    // Determine starting directory
    let start_dir = env::args()
        .nth(1)
        .map(Into::into)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| ".".into()));

    // Terminal setup — render to stderr so stdout stays clean for the result
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let selected = run(&mut terminal, start_dir);

    // Terminal teardown — always runs
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Print selected directory so a shell wrapper can cd to it
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

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        // Poll for events with a 50ms timeout (keeps UI responsive)
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                let action = handle_key(key, &app.mode);
                app.update(action);
            }
        }

        if app.should_quit {
            return Ok(app.selected_path);
        }
    }
}
