import { script } from "./shellcode/script.js"
import { Bash, Fish, PowerShell, WinCmd, Zsh } from "./shellcode/shells.js"
import describe from "./describe.js"
import testJavaVersion from "./shellcode/test-java-version.js"

// `fjm install --latest` now resolves against the fixture Adoptium-shaped
// proxy server (see `tests/proxy-server`), picking the highest available
// major (23, non-LTS in the fixture set) instead of asserting stub-error
// text.
for (const shell of [Bash, Zsh, Fish, PowerShell, WinCmd]) {
  describe(shell, () => {
    test(`installing latest resolves against the fixture server`, async () => {
      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["install", "--latest", "--use"]))
        .then(testJavaVersion(shell, "23.0.1"))
        .takeSnapshot(shell)
        .execute(shell)
    })
  })
}
