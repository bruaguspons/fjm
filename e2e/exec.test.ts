import { script, fjmDirForCurrentTest } from "./shellcode/script.js"
import { Bash, Fish, PowerShell, WinCmd, Zsh } from "./shellcode/shells.js"
import { writeFile } from "node:fs/promises"
import { join } from "node:path"
import testCwd from "./shellcode/test-cwd.js"
import describe from "./describe.js"
import seedJdkInstall from "./shellcode/seed-jdk-install.js"

for (const shell of [Bash, Zsh, Fish, PowerShell, WinCmd]) {
  describe(shell, () => {
    test("`exec` usage", async () => {
      seedJdkInstall(fjmDirForCurrentTest(), "17.0.2")
      seedJdkInstall(fjmDirForCurrentTest(), "11.0.21")
      seedJdkInstall(fjmDirForCurrentTest(), "21.0.1")
      await writeFile(join(testCwd(), ".java-version"), "17.0.2")

      await script(shell)
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", ["exec", "--", "java", "--version"]),
            `openjdk version "17.0.2"`,
            "version file exec",
          ),
        )
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", [
              "exec",
              "--using=11.0.21",
              "--",
              "java",
              "--version",
            ]),
            `openjdk version "11.0.21"`,
            "exec:11 java --version",
          ),
        )
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", [
              "exec",
              "--using=21.0.1",
              "--",
              "java",
              "--version",
            ]),
            `openjdk version "21.0.1"`,
            "exec:21 java --version",
          ),
        )
        .takeSnapshot(shell)
        .execute(shell)
    })
  })
}
