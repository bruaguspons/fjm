import { script } from "./shellcode/script.js"
import { Bash, Fish, PowerShell, Zsh } from "./shellcode/shells.js"
import describe from "./describe.js"
import getStderr from "./shellcode/get-stderr.js"

// `fjm install` is an unconditional stub today (see PRD.md §10). These
// exercise the install-path log output through the resulting
// `NotImplemented` error, instead of a real download.
for (const shell of [Bash, Zsh, Fish, PowerShell]) {
  describe(shell, () => {
    test(`"quiet" log level`, async () => {
      await script(shell)
        .then(shell.env({ logLevel: "quiet" }))
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", ["install", "17.0.2"]),
            "",
            "fjm install",
          ),
        )
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", ["use", "17.0.2"]),
            "",
            "fjm use",
          ),
        )
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", ["alias", "17.0.2", "something"]),
            "",
            "fjm alias",
          ),
        )
        .takeSnapshot(shell)
        .execute(shell)
    })

    test("error log level", async () => {
      await script(shell)
        .then(shell.env({ logLevel: "error" }))
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", ["install", "17.0.2"]),
            "",
            "fjm install",
          ),
        )
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", ["use", "17.0.2"]),
            "",
            "fjm use",
          ),
        )
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", ["alias", "17.0.2", "something"]),
            "",
            "fjm alias",
          ),
        )
        .then(
          shell.scriptOutputContains(
            getStderr(shell.call("fjm", ["alias", "abcd", "efg"])),
            `"find requested version"`,
          ),
        )
        .takeSnapshot(shell)
        .execute(shell)
    })
  })
}
