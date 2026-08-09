// @ts-check

import { createServer } from "node:http"
import crypto from "node:crypto"
import chalk from "chalk"
import { buildTarGz, buildZip } from "./archives.mjs"

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
 * extracted `java` script needs to actually run on the CI host.
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

/** @param {string} javaVersion */
function fakeJavaScriptContent(javaVersion) {
  const versionLine = `openjdk version "${javaVersion}"`
  if (process.platform === "win32") {
    return Buffer.from(`@echo off\r\necho ${versionLine}\r\n`, "utf-8")
  }
  return Buffer.from(`#!/bin/sh\necho '${versionLine}'\n`, "utf-8")
}

/** @param {{ major: number, semver: string, javaVersion: string }} fixture */
function archiveFor(fixture) {
  const dirName = `jdk-${fixture.semver}`
  const content = fakeJavaScriptContent(fixture.javaVersion)

  if (process.platform === "win32") {
    return {
      name: `OpenJDK${fixture.major}U-jdk_x64_windows_hotspot_${fixture.javaVersion}.zip`,
      bytes: buildZip([{ name: `${dirName}/bin/java.cmd`, content }]),
    }
  }

  return {
    name: `OpenJDK${fixture.major}U-jdk_x64_linux_hotspot_${fixture.javaVersion}.tar.gz`,
    bytes: buildTarGz([
      { name: `${dirName}/bin/java`, content, executable: true },
    ]),
  }
}

const ARCHIVES = new Map(
  FIXTURE_MAJORS.map((fixture) => [fixture.major, archiveFor(fixture)])
)

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
      version: {
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

  console.log(chalk.red.dim(`[proxy] miss: ${pathname} (no fixture for this path)`))
  res.writeHead(404)
  res.end()
})
