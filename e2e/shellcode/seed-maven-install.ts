import { mkdirSync, writeFileSync, chmodSync } from "node:fs"
import path from "node:path"
import { writeStubAs } from "./compile-java-stub.js"

/**
 * Seeds a fake Maven installation directly on disk, bypassing `fjm install
 * --tool maven` entirely — the Maven sibling of `seed-jdk-install.ts`.
 *
 * Lays out `<fjmDir>/maven-versions/v<version>/installation/bin/{mvn,mvn.cmd}`
 * (unix) / `mvn.exe` (Windows), matching what `fjm install --tool maven`
 * would have produced (`ToolKind::Maven.dir_name()` in `src/tool_kind.rs`,
 * `installation_path` in `src/version.rs`).
 *
 * Writes a fake `mvn` that prints a parseable `Apache Maven <version>` line
 * to stdout, matching the seeded version. See `test-maven-version.ts` for
 * the corresponding assertion helper.
 *
 * @param fjmDir The value passed as `FJM_DIR` for the running script (matches `script.ts`'s `fjmDir` config).
 * @param version The Maven version string to seed, e.g. `"3.9.9"` (no `v` prefix).
 * @returns The absolute path to the seeded `installation` directory.
 */
export default async function seedMavenInstall(
  fjmDir: string,
  version: string,
): Promise<string> {
  const installationDir = path.join(fjmDir, "maven-versions", `v${version}`, "installation")
  const binDir = path.join(installationDir, "bin")

  const versionLine = `Apache Maven ${version}`

  if (process.platform === "win32") {
    await writeStubAs(binDir, "mvn.exe", `${versionLine}\n`)
  } else {
    mkdirSync(binDir, { recursive: true })
    const mvnBin = path.join(binDir, "mvn")
    writeFileSync(mvnBin, `#!/bin/sh\necho '${versionLine}'\n`)
    chmodSync(mvnBin, 0o755)
  }

  return installationDir
}
