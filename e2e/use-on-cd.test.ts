import { writeFile, mkdir } from "node:fs/promises"
import { join } from "node:path"
import { script, fjmDirForCurrentTest } from "./shellcode/script.js"
import { Bash, Fish, PowerShell, WinCmd, Zsh } from "./shellcode/shells.js"
import testCwd from "./shellcode/test-cwd.js"
import testJavaVersion from "./shellcode/test-java-version.js"
import describe from "./describe.js"
import seedJdkInstall from "./shellcode/seed-jdk-install.js"

for (const shell of [Bash, Zsh, Fish, PowerShell, WinCmd]) {
  describe(shell, () => {
    test(`use on cd`, async () => {
      seedJdkInstall(fjmDirForCurrentTest(), "17.0.2")
      seedJdkInstall(fjmDirForCurrentTest(), "21.0.1")
      await mkdir(join(testCwd(), "subdir"), { recursive: true })
      await writeFile(join(testCwd(), "subdir", ".java-version"), "21.0.1")
      await script(shell)
        .then(shell.env({ useOnCd: true }))
        .then(shell.call("cd", ["subdir"]))
        .then(testJavaVersion(shell, "21.0.1"))
        .takeSnapshot(shell)
        .execute(shell)
    })

    test(`uses current directory version immediately on env setup`, async () => {
      seedJdkInstall(fjmDirForCurrentTest(), "17.0.2")
      seedJdkInstall(fjmDirForCurrentTest(), "21.0.1")
      await writeFile(join(testCwd(), ".java-version"), "21.0.1")
      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["use", "17.0.2"]))
        .then(shell.env({ useOnCd: true }))
        .then(testJavaVersion(shell, "21.0.1"))
        .execute(shell)
    })

    test(`works after sourcing env twice`, async () => {
      seedJdkInstall(fjmDirForCurrentTest(), "17.0.2")
      seedJdkInstall(fjmDirForCurrentTest(), "21.0.1")
      await mkdir(join(testCwd(), "subdir"), { recursive: true })
      await writeFile(join(testCwd(), "subdir", ".java-version"), "21.0.1")
      await script(shell)
        .then(shell.env({ useOnCd: true }))
        .then(shell.env({ useOnCd: true }))
        .then(shell.call("cd", ["subdir"]))
        .then(testJavaVersion(shell, "21.0.1"))
        .execute(shell)
    })

    test(`doesn't throw on missing version file`, async () => {
      await mkdir(join(testCwd(), "subdir"), { recursive: true })
      await script(shell)
        .then(shell.env({ useOnCd: true }))
        .then(shell.call("cd", ["subdir"]))
        .execute(shell)
    })
  })
}
