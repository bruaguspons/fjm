import { mkdirSync, writeFileSync, chmodSync } from "node:fs"
import path from "node:path"
import { writeStub } from "./compile-java-stub.js"

/**
 * Seeds a fake JDK installation directly on disk, bypassing `fjm install`
 * entirely.
 *
 * `fjm install`/`fjm ls-remote` are unconditional `NotImplemented` stubs
 * today (see `src/downloader.rs`, `src/remote_node_index.rs`, and the
 * design doc for `sdd/e2e-ci-docs-rebrand`), so e2e tests that need to
 * exercise `use`/`current`/`ls`/`alias`/`exec`/`env`/`uninstall`/multishell/
 * use-on-cd end-to-end must seed the install directory the same way `fjm
 * install` would have laid it out: `<fjmDir>/node-versions/<version>/installation/bin/{java,java.exe}`.
 *
 * Writes a fake `java`/`java.exe` that prints a parseable
 * `openjdk version "<version>"` line to stdout, matching the seeded
 * version. See `test-java-version.ts` for the corresponding assertion
 * helper. On Windows this is a real compiled `.exe` (see
 * `compile-java-stub.ts`), since a plain-text file cannot be named
 * `java.exe` and executed there.
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
export default async function seedJdkInstall(fjmDir: string, version: string): Promise<string> {
  const installationDir = path.join(fjmDir, "node-versions", `v${version}`, "installation")
  const binDir = path.join(installationDir, "bin")

  const versionLine = `openjdk version "${version}"`

  // `fjm` adds `installation/bin` to PATH on every platform — see
  // `set_path_for_multishell` in src/commands/env.rs and `Exec::apply` in
  // src/commands/exec.rs — matching the real layout inside an Adoptium
  // archive (`jdk-<version>/bin/java[.exe]`) once the top-level wrapper
  // directory is flattened (`flatten_single_top_level_dir` in
  // src/downloader.rs).
  if (process.platform === "win32") {
    // We seed a real `java.exe` (compiled once via `compile-java-stub.ts`
    // and reused for every seeded install) rather than a `.cmd` script, so
    // it resolves via bare `java` for both `Command::new` and
    // MSYS/Git-Bash's own PATH resolution without depending on `PATHEXT`.
    await writeStub(binDir, `${versionLine}\n`)
  } else {
    mkdirSync(binDir, { recursive: true })
    const javaBin = path.join(binDir, "java")
    writeFileSync(javaBin, `#!/bin/sh\necho '${versionLine}'\n`)
    chmodSync(javaBin, 0o755)
  }

  return installationDir
}
