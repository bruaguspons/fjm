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
      await seedJdkInstall(fjmDirForCurrentTest(), "17.0.2")
      await seedJdkInstall(fjmDirForCurrentTest(), "21.0.1")

      await script(shell)
        .then(shell.env({}))
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", ["current", "--tool", "java"]),
            "none",
            "currently activated version",
          ),
        )
        .then(shell.call("fjm", ["use", "17.0.2"]))
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", ["current", "--tool", "java"]),
            "v17.0.2",
            "currently activated version",
          ),
        )
        .then(shell.call("fjm", ["use", "21.0.1"]))
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", ["current", "--tool", "java"]),
            "v21.0.1",
            "currently activated version",
          ),
        )
        .then(shell.call("fjm", ["use", "system"]))
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", ["current", "--tool", "java"]),
            "system",
            "currently activated version",
          ),
        )
        .takeSnapshot(shell)
        .execute(shell)
    })
  })
}

// `fjm current` with no `--tool` flag now prints every tool's active
// version (one `tool: version` line each), not just Java — see the
// "Multi-Tool Visibility in `current`/`list`" spec requirement.
// `scriptOutputContains` (grep-based, unquoted substrings) isn't wired up
// for WinCmd, so this only runs on the shells that support it — same
// restriction other e2e files already apply for this helper.
for (const shell of [Bash, Zsh, Fish, PowerShell]) {
  describe(shell, () => {
    test(`current with no --tool shows every tool`, async () => {
      await seedJdkInstall(fjmDirForCurrentTest(), "17.0.2")

      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["use", "17.0.2"]))
        .then(shell.scriptOutputContains(shell.call("fjm", ["current"]), "java:"))
        .then(shell.scriptOutputContains(shell.call("fjm", ["current"]), "v17.0.2"))
        .then(shell.scriptOutputContains(shell.call("fjm", ["current"]), "maven:"))
        .takeSnapshot(shell)
        .execute(shell)
    })
  })
}
