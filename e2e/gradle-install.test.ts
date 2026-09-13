import { writeFile } from "node:fs/promises"
import { join } from "node:path"
import { script } from "./shellcode/script.js"
import { Bash, Fish, PowerShell, WinCmd, Zsh } from "./shellcode/shells.js"
import testCwd from "./shellcode/test-cwd.js"
import testGradleVersion from "./shellcode/test-gradle-version.js"
import getStderr from "./shellcode/get-stderr.js"
import describe from "./describe.js"

// `fjm install --tool gradle`/`ls-remote --tool gradle` resolve against the
// fixture Gradle-distribution-service-shaped proxy server (see
// `tests/proxy-server`), so these exercise a real install (download + sha256
// checksum + zip extract) via `ChecksumSource::Embedded`, distinct from
// Maven's `.sha512` sidecar path covered by `maven-install.test.ts`.
for (const shell of [Bash, Zsh, Fish, PowerShell, WinCmd]) {
  describe(shell, () => {
    test(`install --tool gradle resolves a two-component version against the fixture server`, async () => {
      await script(shell)
        .then(shell.env({}))
        .then(
          shell.call("fjm", ["install", "--tool", "gradle", "8.10", "--use"])
        )
        .then(testGradleVersion(shell, "8.10"))
        .takeSnapshot(shell)
        .execute(shell)
    })

    test(`install --tool gradle from .gradle-version`, async () => {
      await writeFile(join(testCwd(), ".gradle-version"), "9.0.0")
      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["install", "--tool", "gradle", "--use"]))
        .then(testGradleVersion(shell, "9.0.0"))
        .takeSnapshot(shell)
        .execute(shell)
    })

    test(`uninstall --tool gradle removes an installed version`, async () => {
      await script(shell)
        .then(shell.env({}))
        .then(
          shell.call("fjm", ["install", "--tool", "gradle", "9.0.0", "--use"])
        )
        .then(shell.call("fjm", ["uninstall", "--tool", "gradle", "9.0.0"]))
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", ["ls", "--tool", "gradle"]),
            "* system",
            "fjm ls --tool gradle"
          )
        )
        .execute(shell)
    })
  })
}

// `scriptOutputContains` isn't implemented for WinCmd
// (see e2e/shellcode/shells/output-contains.ts), matching every other
// assertion-based test file (e.g. `old-versions.test.ts`).
for (const shell of [Bash, Zsh, Fish, PowerShell]) {
  describe(shell, () => {
    test(`installing a Gradle version not present in the fixture server fails clearly`, async () => {
      await script(shell)
        .then(shell.env({}))
        .then(
          shell.scriptOutputContains(
            getStderr(
              shell.call("fjm", ["install", "--tool", "gradle", "1.0.0"])
            ),
            "'no Gradle asset found for version'"
          )
        )
        .takeSnapshot(shell)
        .execute(shell)
    })
  })
}
