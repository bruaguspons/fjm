<h1 align="center">
  Fast Java Manager (<code>fjm</code>)
  <img alt="Amount of downloads" src="https://img.shields.io/github/downloads/bruaguspons/fjm/total.svg?style=flat" />
  <a href="https://github.com/bruaguspons/fjm/actions"><img src="https://img.shields.io/github/actions/workflow/status/bruaguspons/fjm/rust.yml?branch=main&label=workflow" alt="GitHub Actions workflow status" /></a>
</h1>

> 🚀 Fast and simple Java version manager, built in Rust

## Why fjm?

`sdkman` and `nvm` activate versions through a shell **function sourced into your `.bashrc`/`.zshrc`** — it only works in interactive shells that ran that init. `fnm` is a real Rust binary, but activates the same way (`eval "$(fnm env)"` in your shell RC), so it inherits the same limitation.

`fjm` follows the same activation model for this first stage — see [PRD.md](./PRD.md) for the full research behind this decision, and for the shim-based approach (à la `rbenv`/`asdf`) being evaluated as future work to remove the shell-sourcing dependency entirely.

### Prior art

| Project | Activation mechanism | Works without sourcing (cron/CI/`docker exec`)? |
|---|---|---|
| `sdkman-cli` | Bash function sourced in `.bashrc` | ❌ No |
| `nvm` | Bash function (`nvm()`) sourced in `.bashrc`. Doesn't even exist as a command without sourcing. | ❌ No |
| `fnm` | Standalone Rust binary + `eval "$(fnm env)"` in `.bashrc`. Creates a temporary per-shell symlink and prepends its `bin/` to `$PATH` via `std::env::set_var`. | ❌ No (same pattern, more polished) |

**Conclusion:** this isn't an implementation detail `sdkman` got wrong — it's a pattern inherited by the whole `nvm → fnm → sdkman` generation of shell-function/`eval`-based version managers. All of them require the shell to have sourced something at startup.

`fjm` is based on [`fnm`](https://github.com/Schniz/fnm) (Fast Node Manager) — it reuses its proven activation design as the starting point for a Java-focused version manager.

## Status

`fjm install`, `fjm list-remote`, `fjm use --install-if-missing`, and `--lts`/`--latest` resolution
download real JDK releases from the [Adoptium (Eclipse Temurin) API](https://api.adoptium.net),
including checksum verification and archive extraction. `FJM_JDK_DIST_MIRROR` overrides the
Adoptium API base URL if you need to point at a mirror or proxy. LTS selectors (`--lts`,
`lts-<major>`, e.g. `lts-17`) resolve numeric JDK LTS majors — JDK has no release codenames, unlike
Node.

`fjm use`, `fjm current`, `fjm list`, `fjm alias`/`unalias`, `fjm default`, `fjm exec`,
`fjm uninstall`, `fjm env`, and `.java-version` resolution work against installed JDK versions.

## Features

🌎 Cross-platform support (macOS, Windows, Linux)

✨ Single file, easy installation, instant startup

🚀 Built with speed in mind

📂 Works with `.java-version` files

## Installation

```sh
curl -fsSL https://raw.githubusercontent.com/bruaguspons/fjm/main/.ci/install.sh | bash
```

The crate is also published to crates.io (`cargo install fjm`) automatically on every tagged
release; other distribution channels (Homebrew, standalone release binaries) are not set up yet.

### Shell Setup

Set up environment variables by evaluating the output of `fjm env`:

#### Bash

```bash
eval "$(fjm env --use-on-cd --shell bash)"
```

#### Zsh

```zsh
eval "$(fjm env --use-on-cd --shell zsh)"
```

#### Fish shell

```fish
fjm env --use-on-cd --shell fish | source
```

#### PowerShell

```powershell
fjm env --use-on-cd --shell powershell | Out-String | Invoke-Expression
```

Adding a `.java-version` to a project will look like:

```bash
$ java --version
openjdk 21.0.2
$ java --version > .java-version
```

## Daily usage

- `fjm install <version>` — download and extract a JDK (via Adoptium/Temurin), e.g. `fjm install 21` or `fjm install --lts`.
- `fjm use <version>` — activate that version in the current shell (`--install-if-missing` installs it if missing).
- `fjm default <version>` — set the default version for new shells.
- `fjm list` / `fjm list-remote` — installed versions / versions available to download.
- `fjm current` — print the active version.
- With `--use-on-cd` enabled, `cd`-ing into a directory with a `.java-version` file switches the version automatically.

See [docs/commands.md](./docs/commands.md) for the full CLI reference.

## Documentation

- [PRD.md](./PRD.md) — problem statement, architecture decisions, and roadmap (prior-art research is above, in [Why fjm?](#why-fjm)).
- [docs/commands.md](./docs/commands.md) — full CLI reference for every subcommand and flag.
- [docs/configuration.md](./docs/configuration.md) — `fjm env` feature flags (`--use-on-cd`, `--version-file-strategy`) and the JDK distribution mirror override.

## Contributing

PRs welcome :tada:

### Developing:

```sh
# Install Rust
git clone https://github.com/bruaguspons/fjm.git
cd fjm/
cargo build
```

### Running Binary:

```sh
cargo run -- --help # Will behave like `fjm --help`
```

### Running Tests:

```sh
cargo test
```
