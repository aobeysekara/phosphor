use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct Editor {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    parser: Arc<Mutex<vt100::Parser>>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    rows: u16,
    cols: u16,
}

impl Editor {
    pub fn spawn(rows: u16, cols: u16, file: &Path) -> io::Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
            .map_err(|e| io::Error::other(e.to_string()))?;

        let mut cmd = CommandBuilder::new("vim");
        cmd.arg(file.as_os_str());
        for (k, v) in std::env::vars() {
            cmd.env(k, v);
        }
        cmd.env("TERM", "xterm-256color");
        if let Some(parent) = file.parent() {
            if !parent.as_os_str().is_empty() {
                cmd.cwd(parent);
            }
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| io::Error::other(e.to_string()))?;
        drop(pair.slave);

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| io::Error::other(e.to_string()))?;
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| io::Error::other(e.to_string()))?;

        let parser = Arc::new(Mutex::new(vt100::Parser::new(rows, cols, 0)));
        let parser_clone = Arc::clone(&parser);

        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Ok(mut p) = parser_clone.lock() {
                            p.process(&buf[..n]);
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Editor { master: pair.master, writer, parser, child, rows, cols })
    }

    pub fn resize(&mut self, rows: u16, cols: u16) -> io::Result<()> {
        if rows == self.rows && cols == self.cols {
            return Ok(());
        }
        self.master
            .resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
            .map_err(|e| io::Error::other(e.to_string()))?;
        if let Ok(mut p) = self.parser.lock() {
            p.screen_mut().set_size(rows, cols);
        }
        self.rows = rows;
        self.cols = cols;
        Ok(())
    }

    pub fn send_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.writer.write_all(bytes)?;
        self.writer.flush()
    }

    pub fn open_file(&mut self, file: &Path) -> io::Result<()> {
        let path_str = file.to_string_lossy();
        // ESC ensures we're in normal mode, then :e <path><CR>
        let cmd = format!("\x1b:e {}\r", path_str);
        self.send_bytes(cmd.as_bytes())
    }

    pub fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    pub fn parser(&self) -> Arc<Mutex<vt100::Parser>> {
        Arc::clone(&self.parser)
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Translate a ratatui/crossterm `KeyEvent` into the byte sequence a terminal
/// emulator would deliver to a process attached to its pty.
pub fn key_to_bytes(key: KeyEvent) -> Vec<u8> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    let mut out = Vec::new();

    match key.code {
        KeyCode::Char(c) => {
            if alt {
                out.push(0x1b);
            }
            if ctrl {
                let upper = c.to_ascii_uppercase() as u8;
                if (b'A'..=b'_').contains(&upper) {
                    out.push(upper - b'@');
                } else if c == ' ' {
                    out.push(0x00);
                } else {
                    out.push(c as u8);
                }
            } else {
                let mut buf = [0u8; 4];
                out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
            }
        }
        KeyCode::Enter => out.push(b'\r'),
        KeyCode::Backspace => out.push(0x7f),
        KeyCode::Tab => out.push(b'\t'),
        KeyCode::BackTab => out.extend_from_slice(b"\x1b[Z"),
        KeyCode::Esc => out.push(0x1b),
        KeyCode::Up => out.extend_from_slice(b"\x1b[A"),
        KeyCode::Down => out.extend_from_slice(b"\x1b[B"),
        KeyCode::Right => out.extend_from_slice(b"\x1b[C"),
        KeyCode::Left => out.extend_from_slice(b"\x1b[D"),
        KeyCode::Home => out.extend_from_slice(b"\x1b[H"),
        KeyCode::End => out.extend_from_slice(b"\x1b[F"),
        KeyCode::PageUp => out.extend_from_slice(b"\x1b[5~"),
        KeyCode::PageDown => out.extend_from_slice(b"\x1b[6~"),
        KeyCode::Delete => out.extend_from_slice(b"\x1b[3~"),
        KeyCode::Insert => out.extend_from_slice(b"\x1b[2~"),
        KeyCode::F(n) => match n {
            1 => out.extend_from_slice(b"\x1bOP"),
            2 => out.extend_from_slice(b"\x1bOQ"),
            3 => out.extend_from_slice(b"\x1bOR"),
            4 => out.extend_from_slice(b"\x1bOS"),
            5 => out.extend_from_slice(b"\x1b[15~"),
            6 => out.extend_from_slice(b"\x1b[17~"),
            7 => out.extend_from_slice(b"\x1b[18~"),
            8 => out.extend_from_slice(b"\x1b[19~"),
            9 => out.extend_from_slice(b"\x1b[20~"),
            10 => out.extend_from_slice(b"\x1b[21~"),
            11 => out.extend_from_slice(b"\x1b[23~"),
            12 => out.extend_from_slice(b"\x1b[24~"),
            _ => {}
        },
        _ => {}
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(code: KeyCode, mods: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, mods)
    }

    #[test]
    fn plain_char_passes_through() {
        assert_eq!(key_to_bytes(k(KeyCode::Char('a'), KeyModifiers::NONE)), b"a");
    }

    #[test]
    fn ctrl_letter_is_control_code() {
        assert_eq!(
            key_to_bytes(k(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            vec![0x03],
        );
    }

    #[test]
    fn ctrl_space_is_nul() {
        assert_eq!(
            key_to_bytes(k(KeyCode::Char(' '), KeyModifiers::CONTROL)),
            vec![0x00],
        );
    }

    #[test]
    fn alt_letter_is_esc_prefix() {
        assert_eq!(
            key_to_bytes(k(KeyCode::Char('c'), KeyModifiers::ALT)),
            vec![0x1b, b'c'],
        );
    }

    #[test]
    fn enter_is_carriage_return() {
        assert_eq!(key_to_bytes(k(KeyCode::Enter, KeyModifiers::NONE)), vec![b'\r']);
    }

    #[test]
    fn esc_is_one_byte() {
        assert_eq!(key_to_bytes(k(KeyCode::Esc, KeyModifiers::NONE)), vec![0x1b]);
    }

    #[test]
    fn backspace_is_del() {
        assert_eq!(
            key_to_bytes(k(KeyCode::Backspace, KeyModifiers::NONE)),
            vec![0x7f],
        );
    }

    #[test]
    fn tab_is_horizontal_tab() {
        assert_eq!(key_to_bytes(k(KeyCode::Tab, KeyModifiers::NONE)), vec![b'\t']);
    }

    #[test]
    fn arrows_are_csi_sequences() {
        assert_eq!(key_to_bytes(k(KeyCode::Up, KeyModifiers::NONE)), b"\x1b[A");
        assert_eq!(key_to_bytes(k(KeyCode::Down, KeyModifiers::NONE)), b"\x1b[B");
        assert_eq!(key_to_bytes(k(KeyCode::Right, KeyModifiers::NONE)), b"\x1b[C");
        assert_eq!(key_to_bytes(k(KeyCode::Left, KeyModifiers::NONE)), b"\x1b[D");
    }

    #[test]
    fn f1_is_ss3_prefix() {
        assert_eq!(key_to_bytes(k(KeyCode::F(1), KeyModifiers::NONE)), b"\x1bOP");
    }

    #[test]
    fn f5_is_csi_prefix() {
        assert_eq!(
            key_to_bytes(k(KeyCode::F(5), KeyModifiers::NONE)),
            b"\x1b[15~",
        );
    }
}
