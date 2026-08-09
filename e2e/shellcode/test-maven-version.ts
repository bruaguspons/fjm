import { HasCall } from "./shells/cmdCall.js"
import { ScriptLine } from "./shells/types.js"
import { HasExpectCommandOutput } from "./shells/expect-command-output.js"

export default function testMavenVersion<
  S extends HasCall & HasExpectCommandOutput
>(shell: S, version: string): ScriptLine {
  const mvnVersion = shell.call("mvn", ["--version"])
  return shell.hasCommandOutput(
    mvnVersion,
    `Apache Maven ${version}`,
    "maven version"
  )
}
