# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 1.0.0

First release. `fjm` is a Rust CLI JDK version manager: a real Rust binary whose `fjm env` output
you `eval` into your shell RC, plus per-directory `.java-version` files.

### Added

- JDK install/list-remote backed by the [Adoptium (Eclipse Temurin) API](https://api.adoptium.net),
  including checksum verification and archive extraction (`--jdk-dist-mirror`/`FJM_JDK_DIST_MIRROR`
  to override the API base URL).
- `fjm use`, `fjm current`, `fjm list`, `fjm default`, `fjm alias`/`unalias`, `fjm exec`,
  `fjm uninstall`, `fjm completions`.
- `fjm env` shell activation for bash, zsh, fish, and PowerShell, with `--use-on-cd` and
  `--version-file-strategy` (`local`/`recursive`) support.
- `.java-version` file resolution.
