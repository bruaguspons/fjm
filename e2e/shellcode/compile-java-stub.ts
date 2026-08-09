import { spawn } from "node:child_process"
import { createHash } from "node:crypto"
import { existsSync, mkdirSync, copyFileSync, renameSync, writeFileSync } from "node:fs"
import os from "node:os"
import path from "node:path"

/**
 * Windows-only fake JDK executable used by e2e fixtures.
 *
 * A plain-text file cannot be named `java.exe` and executed on Windows
 * (`.exe` requires a real PE binary), so this source is compiled once via
 * bare `rustc` into a real executable and reused as the seeded/decoy `java`
 * everywhere the e2e suite needs one. It reads a fixed-name sibling file
 * (`stub-output.txt`) next to itself and prints its contents verbatim, so
 * the same compiled binary can be copied to many install directories, each
 * with its own sibling file controlling what it prints.
 */
const STUB_SOURCE = `
fn main() {
    let exe = std::env::current_exe().expect("current_exe");
    let dir = exe.parent().expect("parent dir");
    let output = std::fs::read_to_string(dir.join("stub-output.txt")).expect("read stub-output.txt");
    print!("{}", output);
}
`

let cached: Promise<string> | undefined

function cacheDir(): string {
  return path.join(os.tmpdir(), "fjm-e2e-java-stub")
}

function cachedExePath(): string {
  const hash = createHash("sha256").update(STUB_SOURCE).digest("hex")
  return path.join(cacheDir(), `${hash}.exe`)
}

function compileStub(): Promise<string> {
  return new Promise((resolve, reject) => {
    const dir = cacheDir()
    mkdirSync(dir, { recursive: true })

    const target = cachedExePath()
    if (existsSync(target)) {
      resolve(target)
      return
    }

    const tmpBase = path.join(dir, `${process.pid}-${Date.now()}`)
    const sourcePath = `${tmpBase}.rs`
    const outputPath = `${tmpBase}.exe`
    writeFileSync(sourcePath, STUB_SOURCE)

    const rustc = spawn("rustc", [sourcePath, "-o", outputPath])
    let stderr = ""
    rustc.stderr.on("data", (chunk) => {
      stderr += chunk.toString()
    })

    rustc.on("error", (error) => {
      reject(new Error(`Failed to spawn rustc to compile the Windows java stub: ${error.message}`))
    })

    rustc.on("exit", (code) => {
      if (code !== 0) {
        reject(new Error(`rustc failed to compile the Windows java stub (exit code ${code}): ${stderr}`))
        return
      }

      renameSync(outputPath, target)
      resolve(target)
    })
  })
}

/**
 * Compiles (or reuses an already-compiled) fake `java.exe` stub, returning
 * the path to the cached binary. Compilation happens at most once across an
 * entire e2e run, since the cache path is content-addressed and shared
 * across Jest worker processes via disk.
 */
export function getCompiledStubPath(): Promise<string> {
  if (!cached) {
    cached = compileStub()
  }
  return cached
}

/**
 * Copies the compiled stub into `destDir` as `java.exe` and writes the
 * sibling `stub-output.txt` file with `outputText`, which the stub prints
 * verbatim to stdout when invoked.
 */
export async function writeStub(destDir: string, outputText: string): Promise<void> {
  const cachedExe = await getCompiledStubPath()
  mkdirSync(destDir, { recursive: true })
  copyFileSync(cachedExe, path.join(destDir, "java.exe"))
  writeFileSync(path.join(destDir, "stub-output.txt"), outputText)
}
