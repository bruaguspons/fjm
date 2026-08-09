import { script } from "./shellcode/script.js"
import { Bash, Fish, PowerShell, WinCmd, Zsh } from "./shellcode/shells.js"
import describe from "./describe.js"
import testJavaVersion from "./shellcode/test-java-version.js"

// `fjm install --lts` now resolves against the fixture Adoptium-shaped proxy
// server (see `tests/proxy-server`), picking the most recent LTS major (21
// in the fixture set) instead of asserting stub-error text.
for (const shell of [Bash, Zsh, Fish, PowerShell, WinCmd]) {
  describe(shell, () => {
    test(`installing latest lts resolves against the fixture server`, async () => {
      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["install", "--lts", "--use"]))
        .then(testJavaVersion(shell, "21.0.4"))
        .takeSnapshot(shell)
        .execute(shell)
    })
  })
}
