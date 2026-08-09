import { script, fjmDirForCurrentTest } from "./shellcode/script.js"
import { Bash, Fish, PowerShell, Zsh } from "./shellcode/shells.js"
import describe from "./describe.js"
import testJavaVersion from "./shellcode/test-java-version.js"
import seedJdkInstall from "./shellcode/seed-jdk-install.js"

// `fjm install` always fails with `NotImplemented` today (see PRD.md §10 and
// src/downloader.rs), so it can never observe an "already installed"
// installation on disk. This test instead exercises the fixture-seeded
// equivalent: a pre-existing installation on disk (as `fjm install` would
// have produced) is correctly detected and usable by `ls`/`use`, without
// going through `fjm install` at all.
for (const shell of [Bash, Zsh, Fish, PowerShell]) {
  describe(shell, () => {
    test(`detects a pre-existing installation on disk`, async () => {
      seedJdkInstall(fjmDirForCurrentTest(), "17.0.2")

      await script(shell)
        .then(shell.env({}))
        .then(
          shell.scriptOutputContains(shell.call("fjm", ["ls"]), "17.0.2"),
        )
        .then(shell.call("fjm", ["use", "17.0.2"]))
        .then(testJavaVersion(shell, "17.0.2"))
        .takeSnapshot(shell)
        .execute(shell)
    })
  })
}
