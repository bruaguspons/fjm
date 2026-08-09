#!/usr/bin/env node

/// @ts-check

import { execa } from "execa"
import fs from "node:fs"
import cmd from "cmd-ts"
import cmdFs from "cmd-ts/dist/cjs/batteries/fs.js"

const FjmBinaryPath = {
  ...cmdFs.ExistingPath,
  defaultValue() {
    const target = new URL("../target/debug/fjm", import.meta.url)
    if (!fs.existsSync(target)) {
      throw new Error(
        "Can't find debug target, please run `cargo build` or provide a specific binary path"
      )
    }
    return target.pathname
  },
}

const command = cmd.command({
  name: "print-command-docs",
  description: "prints the docs/command.md file with updated contents",
  args: {
    checkForDirty: cmd.flag({
      long: "check",
      description: `Check that file was not changed`,
    }),
    fjmPath: cmd.option({
      long: "binary-path",
      description: "the fjm binary path",
      type: FjmBinaryPath,
    }),
  },
  async handler({ checkForDirty, fjmPath }) {
    const targetFile = new URL("../docs/commands.md", import.meta.url).pathname
    await main(targetFile, fjmPath)
    if (checkForDirty) {
      const gitStatus = await checkGitStatus(targetFile)
      if (gitStatus.state === "dirty") {
        process.exitCode = 1
        console.error(
          "The file has changed. Please re-run `pnpm generate-command-docs`."
        )
        console.error(`hint: The following diff was found:`)
        console.error()
        console.error(gitStatus.diff)
      }
    }
  },
})

cmd.run(cmd.binary(command), process.argv).catch((err) => {
  console.error(err)
  process.exitCode = process.exitCode || 1
})

/**
 * @param {string} targetFile
 * @param {string} fjmPath
 * @returns {Promise<void>}
 */
async function main(targetFile, fjmPath) {
  const stream = fs.createWriteStream(targetFile)

  const { subcommands, text: mainText } = await getCommandHelp(fjmPath)

  await write(stream, line(`fjm`, mainText))

  for (const subcommand of subcommands) {
    const { text: subcommandText } = await getCommandHelp(fjmPath, subcommand)
    await write(stream, "\n" + line(`fjm ${subcommand}`, subcommandText))
  }

  stream.close()

  await execa(`pnpm`, ["prettier", "--write", targetFile])
}

/**
 * @param {import('stream').Writable} stream
 * @param {string} content
 * @returns {Promise<void>}
 */
function write(stream, content) {
  return new Promise((resolve, reject) => {
    stream.write(content, (err) => (err ? reject(err) : resolve()))
  })
}

function line(cmd, text) {
  const cmdCode = "`" + cmd + "`"
  const textCode = "```\n" + text + "\n```"
  return `# ${cmdCode}\n${textCode}`
}

/**
 * @param {string} fjmPath
 * @param {string} [command]
 * @returns {Promise<{ subcommands: string[], text: string }>}
 */
async function getCommandHelp(fjmPath, command) {
  const cmdArg = command ? [command] : []
  const result = await run(fjmPath, [...cmdArg, "--help"])
  const text = result.stdout
  const rows = text.split("\n")
  const headerIndex = rows.findIndex((x) => x.includes("Commands:"))
  /** @type {string[]} */
  const subcommands = []
  if (!command) {
    for (const row of rows.slice(
      headerIndex + 1,
      rows.indexOf("", headerIndex + 1)
    )) {
      const [, word] = row.split(/\s+/)
      if (word && word[0].toLowerCase() === word[0]) {
        subcommands.push(word)
      }
    }
  }
  return {
    subcommands,
    text,
  }
}

/**
 * @param {string[]} args
 * @returns {import('execa').ExecaChildProcess<string>}
 */
function run(fjmPath, args) {
  return execa(fjmPath, args, {
    reject: false,
    stdout: "pipe",
    stderr: "pipe",
  })
}

/**
 * @param {string} targetFile
 * @returns {Promise<{ state: "dirty", diff: string } | { state: "clean" }>}
 */
async function checkGitStatus(targetFile) {
  const { stdout, exitCode } = await execa(
    `git`,
    ["diff", "--color", "--exit-code", targetFile],
    {
      reject: false,
    }
  )
  if (exitCode === 0) {
    return { state: "clean" }
  }
  return { state: "dirty", diff: stdout }
}
