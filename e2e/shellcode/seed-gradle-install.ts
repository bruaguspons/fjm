import { mkdirSync, writeFileSync, chmodSync } from "node:fs"
import path from "node:path"
import { writeStubAs } from "./compile-java-stub.js"

/**
 * Seeds a fake Gradle installation directly on disk, bypassing `fjm install
 * --tool gradle` entirely — the Gradle sibling of `seed-jdk-install.ts` and
 * `seed-maven-install.ts`.
 *
 * Lays out `<fjmDir>/gradle-versions/v<version>/installation/bin/gradle`
 * (unix) / `gradle.exe` (Windows), matching what `fjm install --tool gradle`
 * would have produced (`ToolKind::Gradle.dir_name()` in `src/tool_kind.rs`,
 * `installation_path` in `src/version.rs`).
 *
 * Writes a fake `gradle` that prints a parseable `Gradle <version>` line to
 * stdout, matching the seeded version. See `test-gradle-version.ts` for the
 * corresponding assertion helper.
 *
 * @param fjmDir The value passed as `FJM_DIR` for the running script (matches `script.ts`'s `fjmDir` config).
 * @param version The Gradle version string to seed, e.g. `"8.10.0"` (no `v` prefix).
 * @returns The absolute path to the seeded `installation` directory.
 */
export default async function seedGradleInstall(
  fjmDir: string,
  version: string,
): Promise<string> {
  const installationDir = path.join(fjmDir, "gradle-versions", `v${version}`, "installation")
  const binDir = path.join(installationDir, "bin")

  const versionLine = `Gradle ${version}`

  if (process.platform === "win32") {
    await writeStubAs(binDir, "gradle.exe", `${versionLine}\n`)
  } else {
    mkdirSync(binDir, { recursive: true })
    const gradleBin = path.join(binDir, "gradle")
    writeFileSync(gradleBin, `#!/bin/sh\necho '${versionLine}'\n`)
    chmodSync(gradleBin, 0o755)
  }

  return installationDir
}
