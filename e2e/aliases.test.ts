import { script, fjmDirForCurrentTest } from "./shellcode/script.js"
import { Bash, Fish, PowerShell, Zsh } from "./shellcode/shells.js"
import describe from "./describe.js"
import getStderr from "./shellcode/get-stderr.js"
import testJavaVersion from "./shellcode/test-java-version.js"
import seedJdkInstall from "./shellcode/seed-jdk-install.js"

for (const shell of [Bash, Zsh, Fish, PowerShell]) {
  describe(shell, () => {
    test(`errors when alias is not found`, async () => {
      await script(shell)
        .then(shell.env({ logLevel: "error" }))
        .then(
          shell.scriptOutputContains(
            getStderr(shell.call("fjm", ["use", "made-up-alias"])),
            "'Requested version made-up-alias is not'",
          ),
        )
        .takeSnapshot(shell)
        .execute(shell)
    })

    test(`unalias a version`, async () => {
      await seedJdkInstall(fjmDirForCurrentTest(), "11.0.21")
      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["alias", "11.0.21", "version11"]))
        .then(
          shell.scriptOutputContains(shell.call("fjm", ["ls"]), "version11"),
        )
        .then(shell.call("fjm", ["unalias", "version11"]))
        .then(
          shell.scriptOutputContains(
            getStderr(shell.call("fjm", ["use", "version11"])),
            "'Requested version version11 is not currently installed'",
          ),
        )
        .takeSnapshot(shell)
        .execute(shell)
    })

    test(`unalias errors if alias not found`, async () => {
      await script(shell)
        .then(shell.env({ logLevel: "error" }))
        .then(
          shell.scriptOutputContains(
            getStderr(shell.call("fjm", ["unalias", "lts"])),
            "'Requested alias lts not found'",
          ),
        )
        .takeSnapshot(shell)
        .execute(shell)
    })

    test(`can alias the system java`, async () => {
      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["alias", "system", "my_system"]))
        .then(
          shell.scriptOutputContains(shell.call("fjm", ["ls"]), "my_system"),
        )
        .then(shell.call("fjm", ["alias", "system", "default"]))
        .then(shell.call("fjm", ["alias", "my_system", "my_system2"]))
        .then(
          shell.scriptOutputContains(shell.call("fjm", ["ls"]), "my_system2"),
        )
        .then(
          shell.scriptOutputContains(
            shell.call("fjm", ["use", "my_system"]),
            "'Bypassing fjm'",
          ),
        )
        .then(shell.call("fjm", ["unalias", "my_system"]))
        .then(
          shell.scriptOutputContains(
            getStderr(shell.call("fjm", ["use", "my_system"])),
            "'Requested version my_system is not currently installed'",
          ),
        )
        .takeSnapshot(shell)
        .execute(shell)
    })

    test(`aliasing versions`, async () => {
      await seedJdkInstall(fjmDirForCurrentTest(), "11.0.21")
      await seedJdkInstall(fjmDirForCurrentTest(), "17.0.2")
      const installedVersions = shell.call("fjm", ["ls"])
      await script(shell)
        .then(shell.env({}))
        .then(shell.call("fjm", ["alias", "17.0.2", "newer"]))
        .then(shell.call("fjm", ["alias", "11.0.21", "older"]))
        .then(shell.call("fjm", ["default", "older"]))
        .then(
          shell.scriptOutputContains(
            shell.scriptOutputContains(installedVersions, "17.0.2"),
            "newer",
          ),
        )
        .then(
          shell.scriptOutputContains(
            shell.scriptOutputContains(installedVersions, "11.0.21"),
            "older",
          ),
        )
        .then(shell.call("fjm", ["use", "older"]))
        .then(testJavaVersion(shell, "11.0.21"))
        .then(shell.call("fjm", ["use", "newer"]))
        .then(testJavaVersion(shell, "17.0.2"))
        .then(shell.call("fjm", ["use", "default"]))
        .then(testJavaVersion(shell, "11.0.21"))
        .takeSnapshot(shell)
        .execute(shell)
    })
  })
}
