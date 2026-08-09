---
"fjm": patch
---

`fjm` (Fast Java Manager) is a Rust CLI JDK version manager, following the same activation model as `fnm`/`nvm`/`sdkman`: a real Rust binary whose `fjm env` output you `eval` into your shell RC, plus per-directory `.java-version` files.

- JDK install/list-remote backed by the [Adoptium (Eclipse Temurin) API](https://api.adoptium.net), including checksum verification and archive extraction (`--jdk-dist-mirror`/`FJM_JDK_DIST_MIRROR` to override the API base URL).
- `fjm use`, `fjm current`, `fjm list`, `fjm default`, `fjm alias`/`unalias`, `fjm exec`, `fjm uninstall`, `fjm completions`.
- `fjm env` shell activation for bash, zsh, fish, and PowerShell, with `--use-on-cd` and `--version-file-strategy` (`local`/`recursive`) support.
- `.java-version` file resolution.
