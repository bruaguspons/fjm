import { script, fjmDirForCurrentTest } from "./shellcode/script.js"
import { Bash, Fish, PowerShell, WinCmd, Zsh } from "./shellcode/shells.js"
import describe from "./describe.js"
import seedJdkInstall from "./shellcode/seed-jdk-install.js"

// `fjm current` displays a semver-parsed version through `Version`'s
// `Display` impl, which always prepends `v` (see `src/version.rs`) — the
// same convention as `fjm ls`. Input given to `fjm use`/`.java-version`
// stays bare (`17.0.2`), but the printed "currently activated" value is
// `v17.0.2`.
for (const shell of [Bash, Zsh, Fish, PowerShell, WinCmd]) {
  describe(shell, () => {
    test(`current returns the current JDK version set in fjm`, async () => {
      seedJdkInstall(fjmDirForCurrentTest(), "17.0.2")
      seedJdkInstall(fjmDirForCurrentTest(), "21.0.1")

      await script(shell)
        .then(shell.env({}))
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", ["current"]),
            "none",
            "currently activated version",
          ),
        )
        .then(shell.call("fjm", ["use", "17.0.2"]))
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", ["current"]),
            "v17.0.2",
            "currently activated version",
          ),
        )
        .then(shell.call("fjm", ["use", "21.0.1"]))
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", ["current"]),
            "v21.0.1",
            "currently activated version",
          ),
        )
        .then(shell.call("fjm", ["use", "system"]))
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", ["current"]),
            "system",
            "currently activated version",
          ),
        )
        .takeSnapshot(shell)
        .execute(shell)
    })
  })
}
