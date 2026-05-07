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
Type "cd /Users/indi/dev/ai/claude/phosphor"
Enter
Sleep 200ms
Show

Type "phosphor"
Enter
Sleep 1500ms

Screenshot screenshots/01-browser.png

# Move the cursor down a few rows so the orange highlight is on a file rather
# than the top entry.
Down@120ms 5
Sleep 600ms

Screenshot screenshots/02-selected.png

# Walk back up to src/, drill in, and open main.rs in the right-pane vim.
Up@120ms 2
Sleep 200ms
Right
Sleep 700ms

# In src/: cursor at .claude (0). Skip past .claude to main.rs.
Down@120ms 5
Right
Sleep 4000ms

Screenshot screenshots/03-split-vim.png

# Tear down cleanly.
Type ":q"
Enter
Sleep 600ms
Type "q"
Sleep 300ms
