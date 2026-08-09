import { writeFile } from "node:fs/promises"
import { join } from "node:path"
import { script, fjmDirForCurrentTest } from "./shellcode/script.js"
import { Bash, Fish, PowerShell, WinCmd, Zsh } from "./shellcode/shells.js"
import testCwd from "./shellcode/test-cwd.js"
import testJavaVersion from "./shellcode/test-java-version.js"
import seedJdkInstall from "./shellcode/seed-jdk-install.js"
import describe from "./describe.js"

// `fjm install`/`ls-remote` now resolve against the fixture Adoptium-shaped
// proxy server (see `tests/proxy-server`), so these exercise a real install
// (download + checksum + extract) instead of asserting stub-error text.
for (const shell of [Bash, Zsh, Fish, PowerShell, WinCmd]) {
  describe(shell, () => {
    test(`install resolves a major against the fixture server`, async () => {
      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["install", "17", "--use"]))
        .then(testJavaVersion(shell, "17.0.9"))
        .takeSnapshot(shell)
        .execute(shell)
    })

    test(`install from .java-version`, async () => {
      await writeFile(join(testCwd(), ".java-version"), "17")
      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["install", "--use"]))
        .then(testJavaVersion(shell, "17.0.9"))
        .takeSnapshot(shell)
        .execute(shell)
    })
  })
}

for (const shell of [Bash, Zsh, Fish, PowerShell, WinCmd]) {
  describe(shell, () => {
    // Fixture-seeded: bypasses `fjm install` and writes the install
    // directory directly, so `use`/`current`-facing behavior can be
    // exercised end-to-end without a real download.
    test(`use a fixture-seeded install`, async () => {
      await seedJdkInstall(fjmDirForCurrentTest(), "17.0.2")
      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["use", "17.0.2"]))
        .then(testJavaVersion(shell, "17.0.2"))
        .takeSnapshot(shell)
        .execute(shell)
    })

    test(`.java-version`, async () => {
      await seedJdkInstall(fjmDirForCurrentTest(), "17.0.2")
      await writeFile(join(testCwd(), ".java-version"), "17.0.2")
      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["use"]))
        .then(testJavaVersion(shell, "17.0.2"))
        .takeSnapshot(shell)
        .execute(shell)
    })

    test(`resolves partial semver`, async () => {
      await seedJdkInstall(fjmDirForCurrentTest(), "21.0.1")
      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["use", "21"]))
        .then(testJavaVersion(shell, "21.0.1"))
        .takeSnapshot(shell)
        .execute(shell)
    })

    test("`fjm ls` with nothing installed", async () => {
      await script(shell)
        .then(shell.env({}))
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", ["ls"]),
            "* system",
            "fjm ls",
          ),
        )
        .takeSnapshot(shell)
        .execute(shell)
    })
  })
}
