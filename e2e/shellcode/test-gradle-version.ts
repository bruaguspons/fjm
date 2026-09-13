import { HasCall } from "./shells/cmdCall.js"
import { ScriptLine } from "./shells/types.js"
import { HasExpectCommandOutput } from "./shells/expect-command-output.js"

export default function testGradleVersion<
  S extends HasCall & HasExpectCommandOutput
>(shell: S, version: string): ScriptLine {
  const gradleVersion = shell.call("gradle", ["--version"])
  return shell.hasCommandOutput(
    gradleVersion,
    `Gradle ${version}`,
    "gradle version"
  )
}
