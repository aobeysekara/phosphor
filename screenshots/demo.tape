# vhs tape that captures phosphor in three states.
#   1. screenshots/01-browser.png — main browser with the orange logo
#   2. screenshots/02-selected.png — a file selected in the list
#   3. screenshots/03-split-vim.png — split-pane layout with vim live in the right pane
#
# Run from the phosphor repo root:  vhs screenshots/demo.tape

Output screenshots/demo.gif

Set Shell "zsh"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1400
Set Height 900
Set Padding 20
Set Theme "Dracula"

Hide
Type "clear"
Enter
Sleep 200ms
Show

Type "phosphor"
Enter
Sleep 1500ms

# Top of list (screenshots/). Park the cursor here in case vhs flushed any
# spurious keystrokes during startup.
Up@80ms 5
Sleep 200ms

Screenshot screenshots/01-browser.png

# Move down to a real file so the orange highlight lands on Cargo.lock.
# Entries: screenshots/, src/, .gitignore, Cargo.lock, Cargo.toml, LICENSE, README.md
Down@120ms 3
Sleep 600ms

Screenshot screenshots/02-selected.png

# Walk back up to src/, drill in, and open main.rs in the right-pane vim.
Up@120ms 2
Sleep 200ms
Right
Sleep 700ms

# In src/: app.rs (0), editor.rs (1), fuzzy.rs (2), input.rs (3), main.rs (4).
Down@120ms 4
Right
Sleep 4000ms

Screenshot screenshots/03-split-vim.png

# Tear down cleanly.
Type ":q"
Enter
Sleep 600ms
Type "q"
Sleep 300ms
