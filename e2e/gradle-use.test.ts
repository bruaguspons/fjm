import { writeFile } from "node:fs/promises"
import { join } from "node:path"
import { script, fjmDirForCurrentTest } from "./shellcode/script.js"
import { Bash, Fish, PowerShell, WinCmd, Zsh } from "./shellcode/shells.js"
import describe from "./describe.js"
import testCwd from "./shellcode/test-cwd.js"
import seedJdkInstall from "./shellcode/seed-jdk-install.js"
import seedGradleInstall from "./shellcode/seed-gradle-install.js"
import testJavaVersion from "./shellcode/test-java-version.js"
import testGradleVersion from "./shellcode/test-gradle-version.js"

// `fjm use --tool gradle` activates the Gradle slot (GRADLE_HOME + PATH)
// independently of any Java slot — see the "Per-Tool Activation Slot" and
// "Gradle Environment Activation" spec scenarios. Mirrors
// `e2e/maven-use.test.ts`.
for (const shell of [Bash, Zsh, Fish, PowerShell, WinCmd]) {
  describe(shell, () => {
    test(`use --tool gradle activates Gradle without disturbing Java`, async () => {
      await seedJdkInstall(fjmDirForCurrentTest(), "17.0.2")
      await seedGradleInstall(fjmDirForCurrentTest(), "8.10.0")

      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["use", "17.0.2"]))
        .then(testJavaVersion(shell, "17.0.2"))
        .then(shell.call("fjm", ["use", "--tool", "gradle", "8.10.0"]))
        .then(testGradleVersion(shell, "8.10.0"))
        // Java's slot must still resolve after activating Gradle.
        .then(testJavaVersion(shell, "17.0.2"))
        .takeSnapshot(shell)
        .execute(shell)
    })

    test(`use --tool gradle works with no Java version active`, async () => {
      await seedGradleInstall(fjmDirForCurrentTest(), "8.10.0")

      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["use", "--tool", "gradle", "8.10.0"]))
        .then(testGradleVersion(shell, "8.10.0"))
        .takeSnapshot(shell)
        .execute(shell)
    })

    // `.java-version` and `.gradle-version` in the same directory resolve
    // independently — see the "Independent co-resolution" spec scenario.
    test(`.java-version and .gradle-version coexist in one directory`, async () => {
      await seedJdkInstall(fjmDirForCurrentTest(), "17.0.2")
      await seedGradleInstall(fjmDirForCurrentTest(), "8.10.0")
      await writeFile(join(testCwd(), ".java-version"), "17.0.2")
      await writeFile(join(testCwd(), ".gradle-version"), "8.10.0")

      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["use"]))
        .then(testJavaVersion(shell, "17.0.2"))
        .then(shell.call("fjm", ["use", "--tool", "gradle"]))
        .then(testGradleVersion(shell, "8.10.0"))
        .takeSnapshot(shell)
        .execute(shell)
    })

    test(`alias/unalias, default, and current work for --tool gradle`, async () => {
      await seedGradleInstall(fjmDirForCurrentTest(), "8.10.0")

      await script(shell)
        .then(shell.env({}))
        .then(
          shell.call("fjm", ["alias", "8.10.0", "my-gradle", "--tool", "gradle"])
        )
        .then(shell.call("fjm", ["default", "8.10.0", "--tool", "gradle"]))
        .then(shell.call("fjm", ["use", "--tool", "gradle", "8.10.0"]))
        .then(testGradleVersion(shell, "8.10.0"))
        .then(
          shell.call("fjm", ["unalias", "my-gradle", "--tool", "gradle"])
        )
        .then(
          shell.hasCommandOutput(
            shell.call("fjm", ["current", "--tool", "gradle"]),
            "v8.10.0",
            "fjm current --tool gradle"
          )
        )
        .takeSnapshot(shell)
        .execute(shell)
    })
  })
}
