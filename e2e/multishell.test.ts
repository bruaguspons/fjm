import { script, fjmDirForCurrentTest } from "./shellcode/script.js"
import { Bash, Fish, PowerShell, Zsh } from "./shellcode/shells.js"
import testJavaVersion from "./shellcode/test-java-version.js"
import describe from "./describe.js"
import seedJdkInstall from "./shellcode/seed-jdk-install.js"

for (const shell of [Bash, Zsh, Fish, PowerShell]) {
  describe(shell, () => {
    test(`multishell changes don't affect parent`, async () => {
      seedJdkInstall(fjmDirForCurrentTest(), "17.0.2")
      seedJdkInstall(fjmDirForCurrentTest(), "21.0.1")

      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["use", "17.0.2"]))
        .then(
          shell.inSubShell(
            script(shell)
              .then(shell.env({}))
              .then(shell.call("fjm", ["use", "21.0.1"]))
              .then(testJavaVersion(shell, "21.0.1"))
              .asLine(),
          ),
        )
        .then(testJavaVersion(shell, "17.0.2"))
        .takeSnapshot(shell)
        .execute(shell)
    })
  })
}
