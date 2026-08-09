import { writeFile } from "node:fs/promises"
import { join } from "node:path"
import { script } from "./shellcode/script.js"
import { Bash, Fish, PowerShell, WinCmd, Zsh } from "./shellcode/shells.js"
import testCwd from "./shellcode/test-cwd.js"
import testMavenVersion from "./shellcode/test-maven-version.js"
import describe from "./describe.js"

// `fjm install --tool maven`/`ls-remote --tool maven` resolve against the
// fixture Maven Central-shaped proxy server (see `tests/proxy-server`), so
// these exercise a real install (download + `.sha512` sidecar checksum +
// extract) via `ChecksumSource::Sidecar`, distinct from Adoptium's embedded
// checksum path covered by `basic.test.ts`.
for (const shell of [Bash, Zsh, Fish, PowerShell, WinCmd]) {
  describe(shell, () => {
    test(`install --tool maven resolves against the fixture server`, async () => {
      await script(shell)
        .then(shell.env({}))
        .then(
          shell.call("fjm", ["install", "--tool", "maven", "3.9.9", "--use"])
        )
        .then(testMavenVersion(shell, "3.9.9"))
        .takeSnapshot(shell)
        .execute(shell)
    })

    test(`install --tool maven from .maven-version`, async () => {
      await writeFile(join(testCwd(), ".maven-version"), "3.9.9")
      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["install", "--tool", "maven", "--use"]))
        .then(testMavenVersion(shell, "3.9.9"))
        .takeSnapshot(shell)
        .execute(shell)
    })

    test(`uninstall --tool maven removes an installed version`, async () => {
      await script(shell)
        .then(shell.env({}))
        .then(
          shell.call("fjm", ["install", "--tool", "maven", "3.8.8", "--use"])
        )
        .then(shell.call("fjm", ["uninstall", "--tool", "maven", "3.8.8"]))
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", ["ls", "--tool", "maven"]),
            "* system",
            "fjm ls --tool maven"
          )
        )
        .execute(shell)
    })
  })
}
