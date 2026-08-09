import { script, fjmDirForCurrentTest } from "./shellcode/script.js"
import { Bash, Fish, PowerShell, Zsh } from "./shellcode/shells.js"
import describe from "./describe.js"
import seedJdkInstall from "./shellcode/seed-jdk-install.js"

for (const shell of [Bash, Zsh, Fish, PowerShell]) {
  describe(shell, () => {
    test(`uninstalls a version`, async () => {
      await seedJdkInstall(fjmDirForCurrentTest(), "17.0.2")
      await script(shell)
        .then(shell.call("fjm", ["alias", "17.0.2", "hello"]))
        .then(
          shell.scriptOutputContains(
            shell.scriptOutputContains(shell.call("fjm", ["ls"]), "17.0.2"),
            "hello",
          ),
        )
        .then(shell.call("fjm", ["uni", "hello"]))
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", ["ls", "--tool", "java"]),
            "* system",
            "fjm ls",
          ),
        )
        .takeSnapshot(shell)
        .execute(shell)
    })
  })
}
