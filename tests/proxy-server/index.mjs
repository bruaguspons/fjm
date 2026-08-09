// @ts-check

import { createServer } from "node:http"
import crypto from "node:crypto"
import { readFileSync } from "node:fs"
import chalk from "chalk"
import { buildTarGz, buildZip } from "./archives.mjs"
import { getCompiledStubPath } from "../../e2e/shellcode/java-stub-compiler.mjs"

/**
 * Fixture Maven Central-shaped endpoints, alongside the Adoptium ones below:
 * `GET /org/apache/maven/apache-maven/maven-metadata.xml` (used by `fjm
 * ls-remote --tool maven`/`fjm install --tool maven <version>` to resolve
 * `latest`/major-only requests) and
 * `GET /org/apache/maven/apache-maven/:version/apache-maven-:version-bin.{tar.gz,zip}`
 * plus its `.sha512` sidecar (used by the actual download/checksum step —
 * see `src/remote_maven_index.rs`/`src/downloader.rs`'s `ChecksumSource::Sidecar`).
 */
const MAVEN_FIXTURE_VERSIONS = ["3.8.8", "3.9.9"]

/** @param {string} version */
function mavenArchiveName(version) {
  return process.platform === "win32"
    ? `apache-maven-${version}-bin.zip`
    : `apache-maven-${version}-bin.tar.gz`
}

/** @param {string} version */
async function buildMavenArchive(version) {
  const dirName = `apache-maven-${version}`
  const versionLine = `Apache Maven ${version}`

  if (process.platform === "win32") {
    const stubExePath = await getCompiledStubPath()
    return buildZip([
      { name: `${dirName}/bin/mvn.exe`, content: readFileSync(stubExePath) },
      {
        name: `${dirName}/bin/stub-output.txt`,
        content: Buffer.from(`${versionLine}\n`, "utf-8"),
      },
    ])
  }

  return buildTarGz([
    {
      name: `${dirName}/bin/mvn`,
      content: Buffer.from(`#!/bin/sh\necho '${versionLine}'\n`, "utf-8"),
      executable: true,
    },
  ])
}

/** @type {Map<string, Buffer>} */
const MAVEN_ARCHIVES = new Map()

export async function mavenReady() {
  for (const [version, bytes] of await Promise.all(
    MAVEN_FIXTURE_VERSIONS.map(async (version) => [version, await buildMavenArchive(version)]),
  )) {
    MAVEN_ARCHIVES.set(version, bytes)
  }
}

function mavenMetadataBody() {
  const versionsXml = MAVEN_FIXTURE_VERSIONS.map((v) => `      <version>${v}</version>`).join(
    "\n",
  )
  const latest = MAVEN_FIXTURE_VERSIONS[MAVEN_FIXTURE_VERSIONS.length - 1]
  return `<?xml version="1.0" encoding="UTF-8"?>
<metadata>
  <groupId>org.apache.maven</groupId>
  <artifactId>apache-maven</artifactId>
  <versioning>
    <latest>${latest}</latest>
    <release>${latest}</release>
    <versions>
${versionsXml}
    </versions>
    <lastUpdated>20241030090000</lastUpdated>
  </versioning>
</metadata>`
}

/**
 * Real-shaped Adoptium/Temurin API v3 fixture server used by the e2e suite.
 *
 * Serves `/v3/info/available_releases`,
 * `/v3/assets/latest/:major/hotspot` (used by `fjm install --latest`/`--lts`)
 * and `/v3/assets/feature_releases/:major/ga` (used by ordinary `fjm
 * install` requests, so exact-patch requests resolve against the real
 * per-major release list instead of always installing the latest) with the
 * same JSON shape as the real `https://api.adoptium.net` (captured live and
 * used to seed `src/remote_node_index.rs`'s unit test fixtures), plus real,
 * extractable archive bytes at each asset's `package.link` — a `.tar.gz` on
 * unix or a `.zip` on Windows, matching `fjm`'s own archive-format dispatch
 * in `src/downloader.rs`. The archive always targets the *running* platform
 * (not the `os`/`architecture` query params), since this is a mock and the
 * extracted `java` binary needs to actually run on the CI host — a real
 * compiled `java.exe` stub on Windows (see `java-stub-compiler.mjs`), since
 * neither `Command::new`/MSYS Bash resolve a bare `java` to a `.cmd`/`.bat`
 * sibling the way `cmd.exe`/PowerShell do.
 *
 * Each fixture major only has a single known patch (see `FIXTURE_MAJORS`),
 * so `feature_releases` only ever has one entry to offer: requesting an
 * exact patch that isn't that one correctly resolves to a "no matching
 * asset" error, just like the real API would for a nonexistent patch.
 */

/** @type {{ major: number, lts: boolean, semver: string, javaVersion: string }[]} */
const FIXTURE_MAJORS = [
  { major: 17, lts: true, semver: "17.0.9+9", javaVersion: "17.0.9" },
  { major: 21, lts: true, semver: "21.0.4+7", javaVersion: "21.0.4" },
  { major: 23, lts: false, semver: "23.0.1+11", javaVersion: "23.0.1" },
]

/** @param {{ major: number, semver: string, javaVersion: string }} fixture */
async function archiveFor(fixture) {
  const dirName = `jdk-${fixture.semver}`
  const versionLine = `openjdk version "${fixture.javaVersion}"`

  if (process.platform === "win32") {
    // Git Bash/MSYS won't resolve a bare `java` to a `.cmd`/`.bat` sibling
    // the way `cmd.exe`/PowerShell do (only `.exe` is auto-appended), so we
    // pack the same compiled `java.exe` stub used for seeded installs (see
    // `java-stub-compiler.mjs`) instead of a batch script.
    const stubExePath = await getCompiledStubPath()
    return {
      name: `OpenJDK${fixture.major}U-jdk_x64_windows_hotspot_${fixture.javaVersion}.zip`,
      bytes: buildZip([
        { name: `${dirName}/bin/java.exe`, content: readFileSync(stubExePath) },
        {
          name: `${dirName}/bin/stub-output.txt`,
          content: Buffer.from(`${versionLine}\n`, "utf-8"),
        },
      ]),
    }
  }

  return {
    name: `OpenJDK${fixture.major}U-jdk_x64_linux_hotspot_${fixture.javaVersion}.tar.gz`,
    bytes: buildTarGz([
      {
        name: `${dirName}/bin/java`,
        content: Buffer.from(`#!/bin/sh\necho '${versionLine}'\n`, "utf-8"),
        executable: true,
      },
    ]),
  }
}

/** @type {Map<number, { name: string, bytes: Buffer }>} */
const ARCHIVES = new Map()

/**
 * Populates `ARCHIVES`. Must be awaited before `server` is listened on —
 * building the Windows fixture involves compiling the `java.exe` stub (see
 * `java-stub-compiler.mjs`), which is async. Kept as an explicit function
 * (rather than top-level `await`) since Jest's `globalSetup` loader
 * `require()`s this module and can't load an ESM graph with top-level await.
 */
export async function ready() {
  for (const [major, archive] of await Promise.all(
    FIXTURE_MAJORS.map(async (fixture) => [fixture.major, await archiveFor(fixture)]),
  )) {
    ARCHIVES.set(major, archive)
  }
  await mavenReady()
}

function availableReleasesBody() {
  const ltsMajors = FIXTURE_MAJORS.filter((f) => f.lts).map((f) => f.major)
  return JSON.stringify({
    available_lts_releases: ltsMajors,
    available_releases: FIXTURE_MAJORS.map((f) => f.major),
    most_recent_feature_release: Math.max(...FIXTURE_MAJORS.map((f) => f.major)),
    most_recent_lts: Math.max(...ltsMajors),
    tip_version: Math.max(...FIXTURE_MAJORS.map((f) => f.major)),
  })
}

/** @param {number} major */
function assetsLatestBody(major, downloadPath) {
  const fixture = FIXTURE_MAJORS.find((f) => f.major === major)
  if (!fixture) return "[]"

  const archive = ARCHIVES.get(major)
  if (!archive) return "[]"

  const checksum = crypto.createHash("sha256").update(archive.bytes).digest("hex")

  return JSON.stringify([
    {
      binary: {
        architecture: "x64",
        image_type: "jdk",
        jvm_impl: "hotspot",
        os: process.platform === "win32" ? "windows" : "linux",
        package: {
          checksum,
          link: `http://localhost:8080${downloadPath}`,
          name: archive.name,
          size: archive.bytes.length,
        },
      },
      release_name: `jdk-${fixture.semver}`,
      vendor: "eclipse",
      version: {
        major: fixture.major,
        openjdk_version: fixture.semver,
        semver: fixture.semver,
      },
    },
  ])
}

/**
 * @param {number} major
 * @param {number} page
 */
function assetsFeatureReleasesBody(major, page, downloadPath) {
  // Only page 0 ever has data: every fixture major has exactly one known
  // release, so later pages are always empty, matching how `resolve_asset`
  // stops paginating once a short page comes back.
  if (page > 0) return "[]"

  const fixture = FIXTURE_MAJORS.find((f) => f.major === major)
  if (!fixture) return "[]"

  const archive = ARCHIVES.get(major)
  if (!archive) return "[]"

  const checksum = crypto.createHash("sha256").update(archive.bytes).digest("hex")

  return JSON.stringify([
    {
      binaries: [
        {
          architecture: "x64",
          image_type: "jdk",
          jvm_impl: "hotspot",
          os: process.platform === "win32" ? "windows" : "linux",
          package: {
            checksum,
            link: `http://localhost:8080${downloadPath}`,
            name: archive.name,
            size: archive.bytes.length,
          },
        },
      ],
      release_name: `jdk-${fixture.semver}`,
      release_type: "ga",
      vendor: "eclipse",
      version_data: {
        major: fixture.major,
        openjdk_version: fixture.semver,
        semver: fixture.semver,
      },
    },
  ])
}

export const server = createServer((req, res) => {
  const url = new URL(req.url ?? "/", "http://localhost:8080")
  const pathname = url.pathname

  if (pathname === "/v3/info/available_releases") {
    console.log(chalk.green.dim(`[proxy] serving available_releases fixture`))
    res.writeHead(200, { "content-type": "application/json" })
    res.end(availableReleasesBody())
    return
  }

  const assetsMatch = pathname.match(/^\/v3\/assets\/latest\/(\d+)\/hotspot$/)
  if (assetsMatch) {
    const major = Number(assetsMatch[1])
    console.log(chalk.green.dim(`[proxy] serving assets/latest fixture for major ${major}`))
    res.writeHead(200, { "content-type": "application/json" })
    res.end(assetsLatestBody(major, `/download/${major}`))
    return
  }

  const featureReleasesMatch = pathname.match(
    /^\/v3\/assets\/feature_releases\/(\d+)\/ga$/,
  )
  if (featureReleasesMatch) {
    const major = Number(featureReleasesMatch[1])
    const page = Number(url.searchParams.get("page") ?? "0")
    console.log(
      chalk.green.dim(
        `[proxy] serving assets/feature_releases fixture for major ${major} (page ${page})`,
      ),
    )
    res.writeHead(200, { "content-type": "application/json" })
    res.end(assetsFeatureReleasesBody(major, page, `/download/${major}`))
    return
  }

  const downloadMatch = pathname.match(/^\/download\/(\d+)$/)
  if (downloadMatch) {
    const major = Number(downloadMatch[1])
    const archive = ARCHIVES.get(major)
    if (archive) {
      console.log(chalk.green.dim(`[proxy] serving archive bytes for major ${major}`))
      res.writeHead(200, { "content-type": "application/octet-stream" })
      res.end(archive.bytes)
      return
    }
  }

  if (pathname === "/org/apache/maven/apache-maven/maven-metadata.xml") {
    console.log(chalk.green.dim(`[proxy] serving maven-metadata.xml fixture`))
    res.writeHead(200, { "content-type": "application/xml" })
    res.end(mavenMetadataBody())
    return
  }

  const mavenArtifactMatch = pathname.match(
    /^\/org\/apache\/maven\/apache-maven\/([^/]+)\/apache-maven-[^/]+-bin\.(tar\.gz|zip)(\.sha512)?$/,
  )
  if (mavenArtifactMatch) {
    const version = mavenArtifactMatch[1]
    const isSidecar = Boolean(mavenArtifactMatch[3])
    const bytes = MAVEN_ARCHIVES.get(version)
    const expectedName = mavenArchiveName(version)

    if (bytes && pathname.includes(expectedName)) {
      if (isSidecar) {
        const digest = crypto.createHash("sha512").update(bytes).digest("hex")
        console.log(chalk.green.dim(`[proxy] serving maven .sha512 sidecar for ${version}`))
        res.writeHead(200, { "content-type": "text/plain" })
        res.end(digest)
        return
      }

      console.log(chalk.green.dim(`[proxy] serving maven archive bytes for ${version}`))
      res.writeHead(200, { "content-type": "application/octet-stream" })
      res.end(bytes)
      return
    }
  }

  console.log(chalk.red.dim(`[proxy] miss: ${pathname} (no fixture for this path)`))
  res.writeHead(404)
  res.end()
})
