# phosphor

A keyboard-driven terminal directory browser with fuzzy search, written in Rust.

Navigates directories and integrates with your shell's `cd` — select a directory and your shell changes to it. Opens files directly in vim.

## Features

- vim-style key bindings (`hjkl`, `g`/`G`)
- Real-time fuzzy search with `nucleo`
- Shows hidden files by default (toggle with `.`)
- Opens files in vim (`l` or `Enter` on a file)
- Shell `cd` integration — select a directory to change to it
- k9s-inspired dark navy and cyan colour scheme

## Installation

Requires Rust (stable). Install via [rustup](https://rustup.rs/).

```bash
cargo install --path .
```

Or build without installing:

```bash
cargo build --release
# binary at ./target/release/phosphor
```

## Shell integration

Add a wrapper function to your shell config so selecting a directory automatically changes to it.

**bash** (`~/.bashrc`):
```bash
p() {
  local dir
  dir=$(phosphor "$@") && [[ -n "$dir" ]] && cd "$dir"
}
```

**zsh** (`~/.zshrc`):
```zsh
p() {
  local dir
  dir=$(phosphor "$@") && [[ -n "$dir" ]] && cd "$dir"
}
```

**fish** (`~/.config/fish/config.fish`):
```fish
function p
  set dir (phosphor $argv)
  and test -n "$dir"
  and cd $dir
end
```

Then run `p` (or `p /some/path`) from your terminal.

## Key bindings

### Normal mode

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `l` / `→` / `Enter` | Enter directory / open file in vim |
| `h` / `←` / `Backspace` | Go to parent directory |
| `~` | Go to home directory |
| `g` | Jump to top |
| `G` | Jump to bottom |
| `.` | Toggle hidden files |
| `/` | Enter search mode |
| `Enter` (on directory) | Select directory and exit (triggers shell `cd`) |
| `q` / `Ctrl-C` | Quit without changing directory |

### Search mode

| Key | Action |
|-----|--------|
| Any character | Append to search query |
| `Backspace` | Delete last character |
| `↑` / `↓` | Navigate filtered results |
| `Enter` | Confirm search, return to normal mode |
| `Esc` | Cancel search and clear filter |

## Dependencies

- [ratatui](https://github.com/ratatui/ratatui) — terminal UI framework
- [nucleo-matcher](https://github.com/helix-editor/nucleo) — fuzzy matching
- [dirs](https://github.com/dirs-dev/dirs-rs) — home directory lookup
- [walkdir](https://github.com/BurntSushi/walkdir) — directory traversal

## Licence

MIT — see [LICENSE](LICENSE).
