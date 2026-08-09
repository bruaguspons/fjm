---
"fjm": patch
---

Fix `fjm env` dropping earlier tool slots from PATH on PowerShell and Windows Command Prompt

Since multi-tool support (Java + Maven) landed, `fjm env` prints one PATH-setting line per active tool slot. On PowerShell and `cmd.exe`, each of those lines read this process's own inherited PATH and rewrote it from scratch, so only the last tool's bin directory survived — every earlier one (including Java's) was silently dropped from the shell's PATH. Bash/Zsh/Fish were unaffected since they already accumulate onto the shell's live `$PATH` instead. PowerShell/cmd now do the same.
