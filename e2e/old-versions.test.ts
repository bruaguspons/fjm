import { script } from "./shellcode/script.js"
import { Bash, Fish, PowerShell, Zsh } from "./shellcode/shells.js"
import describe from "./describe.js"
import getStderr from "./shellcode/get-stderr.js"

// `1.8.0` (pre-JEP223 style but without the legacy `_update` suffix, so it's
// not caught by `is_legacy_update_format`) parses as a normal semver whose
// major is `1`, and resolves against the fixture server's major listing
// like any other version — the fixture set doesn't include major 1, so this
// asserts the real "no matching asset" error instead of a stub.
// `scriptOutputContains` isn't implemented for WinCmd
// (see e2e/shellcode/shells/output-contains.ts), so WinCmd is dropped from
// this loop, matching every other assertion-based test file.
for (const shell of [Bash, Zsh, Fish, PowerShell]) {
  describe(shell, () => {
    test(`installing an old JDK major not present in the fixture server fails clearly`, async () => {
      await script(shell)
        .then(shell.env({}))
        .then(
          shell.scriptOutputContains(
            getStderr(shell.call("fjm", ["install", "1.8.0"])),
            "'no JDK asset found for major 1'",
          ),
        )
        .takeSnapshot(shell)
        .execute(shell)
    })
  })
}
