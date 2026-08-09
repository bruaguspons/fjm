import { script, fjmDirForCurrentTest } from "./shellcode/script.js"
import { Bash, Fish, PowerShell, WinCmd, Zsh } from "./shellcode/shells.js"
import fs from "node:fs/promises"
import path from "node:path"
import describe from "./describe.js"
import testJavaVersion from "./shellcode/test-java-version.js"
import testBinDir from "./shellcode/test-bin-dir.js"
import seedJdkInstall from "./shellcode/seed-jdk-install.js"
import { writeStub } from "./shellcode/compile-java-stub.js"

for (const shell of [Bash, Fish, PowerShell, WinCmd, Zsh]) {
  describe(shell, () => {
    // latest bash breaks this as it seems. gotta find a solution.
    const t = process.platform === "darwin" && shell === Bash ? test.skip : test

    t(`switches to system java`, async () => {
      await writeCustomJava()
      await seedJdkInstall(fjmDirForCurrentTest(), "11.0.21")

      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["use", "11.0.21"]))
        .then(testJavaVersion(shell, "11.0.21"))
        .then(shell.call("fjm", ["use", "system"]))
        .then(
          shell.hasCommandOutput(
            shell.call("java", ["--version"]),
            "custom",
            "system java version",
          ),
        )
        .execute(shell)
    })

    t(`aliasing a system java`, async () => {
      await writeCustomJava()
      await seedJdkInstall(fjmDirForCurrentTest(), "11.0.21")
      const init = script(shell).then(shell.env({}))

      await init
        .then(shell.call("fjm", ["use", "11.0.21"]))
        .then(shell.call("fjm", ["default", "11.0.21"]))
        .execute(shell)

      await init
        .then(testJavaVersion(shell, "11.0.21"))
        .then(shell.call("fjm", ["default", "system"]))
        .execute(shell)

      await init
        .then(
          shell.hasCommandOutput(
            shell.call("java", ["--version"]),
            "custom",
            "system java version",
          ),
        )
        .execute(shell)
    })
  })

  async function writeCustomJava() {
    if (process.platform === "win32") {
      await writeStub(testBinDir(), "custom")
    } else {
      const customJava = path.join(testBinDir(), "java")
      await fs.writeFile(customJava, `#!/bin/bash\n\necho "custom"\n`)
      // set executable
      await fs.chmod(customJava, 0o766)
    }
  }
}
