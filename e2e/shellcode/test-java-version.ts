import { HasCall } from "./shells/cmdCall.js"
import { ScriptLine } from "./shells/types.js"
import { HasExpectCommandOutput } from "./shells/expect-command-output.js"

export default function testJavaVersion<
  S extends HasCall & HasExpectCommandOutput
>(shell: S, version: string): ScriptLine {
  const javaVersion = shell.call("java", ["--version"])
  return shell.hasCommandOutput(
    javaVersion,
    `openjdk version "${version}"`,
    "java version"
  )
}
