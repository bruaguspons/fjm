import { mkdirSync, copyFileSync, writeFileSync } from "node:fs"
import path from "node:path"
import { getCompiledStubPath } from "./java-stub-compiler.mjs"

export { getCompiledStubPath }

/**
 * Copies the compiled stub into `destDir` as `java.exe` and writes the
 * sibling `stub-output.txt` file with `outputText`, which the stub prints
 * verbatim to stdout when invoked. See `java-stub-compiler.mjs` for how the
 * stub itself is compiled and cached.
 */
export async function writeStub(destDir: string, outputText: string): Promise<void> {
  const cachedExe = await getCompiledStubPath()
  mkdirSync(destDir, { recursive: true })
  copyFileSync(cachedExe, path.join(destDir, "java.exe"))
  writeFileSync(path.join(destDir, "stub-output.txt"), outputText)
}
