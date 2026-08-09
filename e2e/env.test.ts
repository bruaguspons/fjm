import { readFile } from "node:fs/promises"
import { join } from "node:path"
import { script } from "./shellcode/script.js"
import { Bash, Fish, PowerShell, WinCmd, Zsh } from "./shellcode/shells.js"
import testCwd from "./shellcode/test-cwd.js"
import describe from "./describe.js"

for (const shell of [Bash, Zsh, Fish, PowerShell, WinCmd]) {
  describe(shell, () => {
    test(`outputs json`, async () => {
      const filename = `file.json`
      await script(shell)
        .then(
          shell.redirectOutput(shell.call("fjm", ["env", "--json"]), {
            output: filename,
          }),
        )
        .takeSnapshot(shell)
        .execute(shell)

      if (shell.currentlySupported()) {
        const file = await readFile(join(testCwd(), filename), "utf8")
        expect(JSON.parse(file)).toEqual({
          FJM_ARCH: expect.any(String),
          FJM_DIR: expect.any(String),
          FJM_LOGLEVEL: "info",
          FJM_MULTISHELL_PATH: expect.any(String),
          FJM_JDK_DIST_MIRROR: expect.any(String),
          FJM_MAVEN_DIST_MIRROR: expect.any(String),
          FJM_VERSION_FILE_STRATEGY: "local",
        })
      }
    })
  })
}
