import { script } from "./shellcode/script.js"
import { Bash, Fish, PowerShell, Zsh } from "./shellcode/shells.js"
import fs from "node:fs/promises"
import path from "node:path"
import describe from "./describe.js"
import testCwd from "./shellcode/test-cwd.js"
import getStderr from "./shellcode/get-stderr.js"

// JDK LTS releases are numeric majors, not codenames (see
// src/lts.rs/src/version.rs): a codename like `lts/dubnium` fails to parse.
// `.java-version` swallows that parse error into "no version in file" (see
// `get_user_version_for_directory_recursive` in src/version_files.rs), so
// this asserts that fallback message instead of a real install. The
// `.nvmrc` dual-file coverage from the original Node-oriented test is
// dropped — fjm has a single version-file format (`.java-version`).
for (const shell of [Bash, Fish, PowerShell, Zsh]) {
  describe(shell, () => {
    test(`installing from a .java-version with an unparseable lts codename falls back to no-version-in-file`, async () => {
      await fs.writeFile(path.join(testCwd(), ".java-version"), `lts/dubnium`)

      await script(shell)
        .then(shell.env({}))
        .then(
          shell.scriptOutputContains(
            getStderr(shell.call("fjm", ["install"])),
            "'find version in dotfiles'",
          ),
        )
        .takeSnapshot(shell)
        .execute(shell)
    })
  })
}
