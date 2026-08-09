import { dedent } from "ts-dedent"
import { define, ScriptLine } from "./types.js"

export type HasExpectCommandOutput = {
  hasCommandOutput(
    script: ScriptLine,
    output: string,
    message: string
  ): ScriptLine
}

export const cmdExpectCommandOutput = {
  bash: define<HasExpectCommandOutput>({
    hasCommandOutput(script, output, message) {
      // `output` may itself contain literal double quotes (e.g. JDK fixture
      // output like `openjdk version "17.0.2"`); single-quoting it keeps
      // those characters literal instead of prematurely closing the
      // surrounding double-quoted string.
      return dedent`
        if [ "$(${script})" != '${output}' ]; then
          echo "Expected ${message} to be '${output}'. Got $(${script})"
          exit 1
        fi
      `
    },
  }),
  fish: define<HasExpectCommandOutput>({
    hasCommandOutput(script, output, message) {
      return dedent`
        set ____test____ (${script})
        if test "$____test____" != '${output}'
          echo "Expected ${message} to be '${output}'. Got $____test____"
          exit 1
        end
      `
    },
  }),
  powershell: define<HasExpectCommandOutput>({
    hasCommandOutput(script, output, message) {
      return dedent`
        if ( "$(${script})" -ne '${output}' ) { echo "Expected ${message} to be '${output}'. Got $(${script})"; exit 1 }
      `
    },
  }),
  wincmd: define<HasExpectCommandOutput>({
    hasCommandOutput(script, output, message) {
      return dedent`
        ${script} | findstr ${output}
        if %errorlevel% neq 0 (
          echo Expected ${message} to be ${output}
          exit 1
        )
      `
    },
  }),
}
