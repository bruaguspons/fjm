import { mkdirSync, writeFileSync, chmodSync } from "node:fs"
import path from "node:path"

/**
 * Seeds a fake JDK installation directly on disk, bypassing `fjm install`
 * entirely.
 *
 * `fjm install`/`fjm ls-remote` are unconditional `NotImplemented` stubs
 * today (see `src/downloader.rs`, `src/remote_node_index.rs`, and the
 * design doc for `sdd/e2e-ci-docs-rebrand`), so e2e tests that need to
 * exercise `use`/`current`/`ls`/`alias`/`exec`/`env`/`uninstall`/multishell/
 * use-on-cd end-to-end must seed the install directory the same way `fjm
 * install` would have laid it out: `<fjmDir>/node-versions/<version>/installation/{bin/java,bin/java.exe}`.
 *
 * Writes a fake `java`/`java.exe` script that prints a parseable
 * `openjdk version "<version>"` line to stdout on `--version`, matching the
 * seeded version. See `test-java-version.ts` for the corresponding
 * assertion helper.
 *
 * The on-disk directory is named `v<version>` (not bare `<version>`):
 * `Version::installation_path()` in `src/version.rs` always resolves a
 * semver install path via `Version::to_string()`, whose `Display` impl
 * prepends `v` (`write!(f, "v{semver}")`). Seeding a bare-`<version>`
 * directory leaves `fjm use` unable to resolve a real symlink target,
 * even though `fjm ls`/
 * `installed_versions::list` parse either form leniently.
 *
 * @param fjmDir The value passed as `FJM_DIR` for the running script (matches `script.ts`'s `fjmDir` config).
 * @param version The JDK version string to seed, e.g. `"17.0.2"` (no `v` prefix).
 * @returns The absolute path to the seeded `installation` directory.
 */
export default function seedJdkInstall(fjmDir: string, version: string): string {
  const installationDir = path.join(fjmDir, "node-versions", `v${version}`, "installation")

  const versionLine = `openjdk version "${version}"`

  if (process.platform === "win32") {
    // On Windows, `fjm` puts the binary directly in the `installation` root
    // (no `bin` subdirectory) — see `set_path_for_multishell` in
    // src/commands/env.rs and the `#[cfg(windows)]` branch in
    // src/commands/exec.rs. A plain-text file also cannot be named
    // `java.exe` and executed on Windows (`.exe` requires a real PE binary),
    // so we use `java.cmd` instead, which Windows resolves through PATHEXT
    // the same way `java.exe` would when `java --version` is invoked
    // without an extension. This is a deliberate deviation from the literal
    // `java.exe` naming in the design doc's open question, since a
    // text-content `.exe` is not runnable.
    mkdirSync(installationDir, { recursive: true })
    const javaBin = path.join(installationDir, "java.cmd")
    writeFileSync(javaBin, `@echo off\r\necho ${versionLine}\r\n`)
  } else {
    // On unix, `fjm` adds `installation/bin` to PATH — see
    // `set_path_for_multishell` in src/commands/env.rs and the
    // `#[cfg(unix)]` branch in src/commands/exec.rs.
    const binDir = path.join(installationDir, "bin")
    mkdirSync(binDir, { recursive: true })
    const javaBin = path.join(binDir, "java")
    writeFileSync(javaBin, `#!/bin/sh\necho '${versionLine}'\n`)
    chmodSync(javaBin, 0o755)
  }

  return installationDir
}
