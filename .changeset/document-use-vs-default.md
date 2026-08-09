---
"fjm": patch
---

Clarify in the README, CLI help text, and `docs/commands.md` that `fjm use` only activates a version for the current shell session and does not persist across terminals — use `fjm default` for that. Also split the Shell Setup section into a "Permanent" subsection (idempotent commands that append `eval "$(fjm env ...)"` to each shell's startup file) and a "Current terminal session only" subsection (the plain `eval` one-liner), so it's explicit which one actually fixes persistence across terminals.
