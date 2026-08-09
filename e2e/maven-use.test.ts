import { writeFile } from "node:fs/promises"
import { join } from "node:path"
import { script, fjmDirForCurrentTest } from "./shellcode/script.js"
import { Bash, Fish, PowerShell, WinCmd, Zsh } from "./shellcode/shells.js"
import describe from "./describe.js"
import testCwd from "./shellcode/test-cwd.js"
import seedJdkInstall from "./shellcode/seed-jdk-install.js"
import seedMavenInstall from "./shellcode/seed-maven-install.js"
import testJavaVersion from "./shellcode/test-java-version.js"
import testMavenVersion from "./shellcode/test-maven-version.js"

// `fjm use --tool maven` activates the Maven slot (MAVEN_HOME + PATH)
// independently of any Java slot — see the "Per-Tool Activation Slot" and
// "Maven active without Java" spec scenarios.
for (const shell of [Bash, Zsh, Fish, PowerShell, WinCmd]) {
  describe(shell, () => {
    test(`use --tool maven activates Maven without disturbing Java`, async () => {
      await seedJdkInstall(fjmDirForCurrentTest(), "17.0.2")
      await seedMavenInstall(fjmDirForCurrentTest(), "3.9.9")

      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["use", "17.0.2"]))
        .then(testJavaVersion(shell, "17.0.2"))
        .then(shell.call("fjm", ["use", "--tool", "maven", "3.9.9"]))
        .then(testMavenVersion(shell, "3.9.9"))
        // Java's slot must still resolve after activating Maven.
        .then(testJavaVersion(shell, "17.0.2"))
        .takeSnapshot(shell)
        .execute(shell)
    })

    test(`use --tool maven works with no Java version active`, async () => {
      await seedMavenInstall(fjmDirForCurrentTest(), "3.9.9")

      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["use", "--tool", "maven", "3.9.9"]))
        .then(testMavenVersion(shell, "3.9.9"))
        .takeSnapshot(shell)
        .execute(shell)
    })

    // `.java-version` and `.maven-version` in the same directory resolve
    // independently — see the "Independent co-resolution" spec scenario.
    test(`.java-version and .maven-version coexist in one directory`, async () => {
      await seedJdkInstall(fjmDirForCurrentTest(), "17.0.2")
      await seedMavenInstall(fjmDirForCurrentTest(), "3.9.9")
      await writeFile(join(testCwd(), ".java-version"), "17.0.2")
      await writeFile(join(testCwd(), ".maven-version"), "3.9.9")

      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["use"]))
        .then(testJavaVersion(shell, "17.0.2"))
        .then(shell.call("fjm", ["use", "--tool", "maven"]))
        .then(testMavenVersion(shell, "3.9.9"))
        .takeSnapshot(shell)
        .execute(shell)
    })
  })
}
