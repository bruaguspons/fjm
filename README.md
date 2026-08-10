<p align="center">
  <img src="./docs/logo.png" alt="fjm logo" width="200">
</p>

<h1 align="center">
  Fast Java Manager (<code>fjm</code>)
</h1>

> 🚀 Fast and simple Java version manager, built in Rust

<p align="center">
  <img src="./docs/fjm.svg" alt="fjm demo" width="600">
</p>

## Why fjm?

`sdkman` and `nvm` activate versions through a shell **function sourced into your `.bashrc`/`.zshrc`** — it only works in interactive shells that ran that init. `fnm` is a real Rust binary, but activates the same way (`eval "$(fnm env)"` in your shell RC), so it inherits the same limitation.

`fjm` follows the same activation model for this first stage — see [PRD.md](./PRD.md) for the full research behind this decision, and for the shim-based approach (à la `rbenv`/`asdf`) being evaluated as future work to remove the shell-sourcing dependency entirely.

This isn't an implementation detail `sdkman` got wrong — it's a pattern inherited by the whole `nvm → fnm → sdkman` generation of shell-function/`eval`-based version managers. All of them require the shell to have sourced something at startup.

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

`fjm` needs to run `eval "$(fjm env ...)"` on every new shell, so it has to live in your shell's
startup file — not just be run once in your current terminal. Otherwise a new terminal starts with
no `fjm`-managed JDK on `PATH` at all, and it looks as if `fjm use`/`fjm default` "didn't persist".

#### Permanent (recommended)

Run one of the commands below once, per shell, to add it to your startup file. Each is
idempotent — safe to re-run without adding a duplicate line.

##### Bash

```bash
grep -qF 'fjm env' ~/.bashrc || echo 'eval "$(fjm env --use-on-cd --shell bash)"' >> ~/.bashrc
```

##### Zsh

```zsh
grep -qF 'fjm env' ~/.zshrc || echo 'eval "$(fjm env --use-on-cd --shell zsh)"' >> ~/.zshrc
```

##### Fish shell

```fish
grep -qF 'fjm env' ~/.config/fish/config.fish 2>/dev/null; or echo 'fjm env --use-on-cd --shell fish | source' >> ~/.config/fish/config.fish
```

##### PowerShell

```powershell
if (-not (Test-Path $PROFILE)) { New-Item -ItemType File -Path $PROFILE -Force | Out-Null }
if (-not (Select-String -Path $PROFILE -Pattern 'fjm env' -Quiet)) {
  Add-Content -Path $PROFILE -Value 'fjm env --use-on-cd --shell powershell | Out-String | Invoke-Expression'
}
```

Then open a new terminal (or `source`/re-import the file you just edited) for it to take effect.

#### Current terminal session only

To try `fjm` out without touching your startup file, just run the `eval` directly — it only
affects the terminal you run it in, and is lost once you close it:

##### Bash

```bash
eval "$(fjm env --use-on-cd --shell bash)"
```

##### Zsh

```zsh
eval "$(fjm env --use-on-cd --shell zsh)"
```

##### Fish shell

```fish
fjm env --use-on-cd --shell fish | source
```

##### PowerShell

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
- `fjm use <version>` — activate that version **for the current shell session only** (`--install-if-missing` installs it if missing). Closing the terminal and opening a new one loses this — it does not change what new shells start with.
- `fjm default <version>` — set the version that **new shells start with**. If you want a version to "stick" across terminals, this is the command you want, not `fjm use`.
- `fjm list` / `fjm list-remote` — installed versions / versions available to download.
- `fjm current` — print the active version.
- With `--use-on-cd` enabled, `cd`-ing into a directory with a `.java-version` file switches the version automatically.

> `fjm use` vs `fjm default`, in short: `use` is "just for this terminal", `default` is "from now on,
> every new terminal". A `.java-version` file in a project directory overrides both, for anyone
> `cd`-ing into it with `--use-on-cd` enabled.

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

## License

GPLv3 — see [LICENSE](./LICENSE).
