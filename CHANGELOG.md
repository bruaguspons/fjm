# Changelog

## 1.2.0

### Minor Changes

- [`6c557da`](https://github.com/bruaguspons/fjm/commit/6c557dac81f01078e7853e7311d7f6bc31530471) Thanks [@bruaguspons](https://github.com/bruaguspons)! - Add `--lts`/`--latest` to `fjm default`, `fjm uninstall`, and `fjm exec --using`, so every command that resolves a version against what's already installed (not just `fjm install`/`fjm use`, which can also download) supports the same shorthand.

  ```sh-session
  $ fjm default --lts
  $ fjm uninstall --latest
  $ fjm exec --lts -- java --version
  ```

  `fjm alias` intentionally keeps its current two-positional-argument shape (`fjm alias <TO_VERSION> <NAME>`) — making `TO_VERSION` optional to fit `--lts`/`--latest` would make it ambiguous with the required `NAME` positional that follows it. Use the existing `lts-latest` alias string there instead (e.g. `fjm alias lts-latest mylabel`).

  Internally, the `--lts`/`--latest` resolution logic (previously duplicated between `install` and `use`) is now centralized in `lts_latest_selector::resolve`.

## 1.1.0

### Minor Changes

- [`c4a38f5`](https://github.com/bruaguspons/fjm/commit/c4a38f576da5d177be4a5734c77d9f4eb9961ad2) Thanks [@bruaguspons](https://github.com/bruaguspons)! - Add Maven version management alongside JDKs, via a new multi-tool activation model.

  `fjm` now manages Java and Maven as independent, parallel tools:

  - `fjm install --tool maven <version>`, `fjm use --tool maven <version>`, `fjm ls-remote --tool maven`, `fjm uninstall --tool maven`, `fjm alias`/`fjm unalias`/`fjm default` all accept `--tool maven` (defaulting to `--tool java` when omitted, for backward compatibility).
  - A `.maven-version` file works alongside `.java-version`, resolved independently in the same directory.
  - `fjm env` now exports `JAVA_HOME` (new, additive) and, when a Maven version is active, `MAVEN_HOME`, alongside the existing PATH/symlink behavior.
  - A new `FJM_MAVEN_DIST_MIRROR` env var / `--maven-dist-mirror` flag lets you override the Maven Central mirror, symmetric to the existing `FJM_JDK_DIST_MIRROR`.
  - Maven downloads are checksum-verified against Maven Central's `.sha512` sidecar files.

  **Behavior change:** `fjm current` and `fjm list` (`fjm ls`), when run without `--tool`, now print every active/installed tool (e.g. `java: 17.0.2` / `maven: 3.9.9`) instead of a single bare JDK version line. Pass `--tool java` to get the previous single-line output back. This is the only breaking output-shape change in this release — JDK-only workflows that always passed an explicit tool, or that don't parse `current`/`list` output, are unaffected.

  Gradle support is intentionally not included — the underlying `ToolKind`/`RemoteVersionIndex` abstraction is designed so it can be added later without reworking the activation model.

- [`9b36978`](https://github.com/bruaguspons/fjm/commit/9b36978281f8db2d01196ab3a965590f1618bfcb) Thanks [@bruaguspons](https://github.com/bruaguspons)! - Add `--lts` and `--latest` flags to `fjm use`, symmetric to the ones already supported by `fjm install`.

  ```sh-session
  $ fjm use --lts --install-if-missing
  $ fjm use --latest --install-if-missing
  ```

  Previously the only way to activate the latest LTS was to pass the alias string directly (`fjm use lts-latest`).

### Patch Changes

- [`90e2976`](https://github.com/bruaguspons/fjm/commit/90e2976414e3326e1f4c5b91fe76c29fa9102393) Thanks [@bruaguspons](https://github.com/bruaguspons)! - the title of the change in a few words.

  If necessary, you can add examples, links to related issues, or any other information that would be helpful for the maintainers and users of the project:

  ```sh-session
  $ fjm whatever --hello world
  the output
  ```

- [`9b36978`](https://github.com/bruaguspons/fjm/commit/9b36978281f8db2d01196ab3a965590f1618bfcb) Thanks [@bruaguspons](https://github.com/bruaguspons)! - Clarify in the README, CLI help text, and `docs/commands.md` that `fjm use` only activates a version for the current shell session and does not persist across terminals — use `fjm default` for that. Also split the Shell Setup section into a "Permanent" subsection (idempotent commands that append `eval "$(fjm env ...)"` to each shell's startup file) and a "Current terminal session only" subsection (the plain `eval` one-liner), so it's explicit which one actually fixes persistence across terminals.

- [`a79c994`](https://github.com/bruaguspons/fjm/commit/a79c9943474b3d3e6ca88fa883f03d1fab478b86) Thanks [@bruaguspons](https://github.com/bruaguspons)! - Fix `fjm env` dropping earlier tool slots from PATH on PowerShell and Windows Command Prompt

  Since multi-tool support (Java + Maven) landed, `fjm env` prints one PATH-setting line per active tool slot. On PowerShell and `cmd.exe`, each of those lines read this process's own inherited PATH and rewrote it from scratch, so only the last tool's bin directory survived — every earlier one (including Java's) was silently dropped from the shell's PATH. Bash/Zsh/Fish were unaffected since they already accumulate onto the shell's live `$PATH` instead. PowerShell/cmd now do the same.

- [`6dc4118`](https://github.com/bruaguspons/fjm/commit/6dc41187694dd5c269425a730358ba90b3423440) Thanks [@bruaguspons](https://github.com/bruaguspons)! - `fjm` (Fast Java Manager) is a Rust CLI JDK version manager, following the same activation model as `fnm`/`nvm`/`sdkman`: a real Rust binary whose `fjm env` output you `eval` into your shell RC, plus per-directory `.java-version` files.

  - JDK install/list-remote backed by the [Adoptium (Eclipse Temurin) API](https://api.adoptium.net), including checksum verification and archive extraction (`--jdk-dist-mirror`/`FJM_JDK_DIST_MIRROR` to override the API base URL).
  - `fjm use`, `fjm current`, `fjm list`, `fjm default`, `fjm alias`/`unalias`, `fjm exec`, `fjm uninstall`, `fjm completions`.
  - `fjm env` shell activation for bash, zsh, fish, and PowerShell, with `--use-on-cd` and `--version-file-strategy` (`local`/`recursive`) support.
  - `.java-version` file resolution.

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
