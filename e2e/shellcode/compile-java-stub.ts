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
  await writeStubAs(destDir, "java.exe", outputText)
}

/**
 * Same as {@link writeStub}, but under an arbitrary executable name — used
 * to seed a fake `mvn.exe` (Maven) alongside `java.exe`, since the compiled
 * stub is generic: it always reads the fixed-name `stub-output.txt` sibling
 * next to itself, regardless of what it's named.
 */
export async function writeStubAs(
  destDir: string,
  exeName: string,
  outputText: string,
): Promise<void> {
  const cachedExe = await getCompiledStubPath()
  mkdirSync(destDir, { recursive: true })
  copyFileSync(cachedExe, path.join(destDir, exeName))
  writeFileSync(path.join(destDir, "stub-output.txt"), outputText)
}
