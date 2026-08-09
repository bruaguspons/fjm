// @ts-check

import zlib from "node:zlib"

/**
 * Minimal, dependency-free USTAR + gzip archive builder, and a minimal
 * dependency-free ZIP (stored, uncompressed) builder — just enough to
 * produce real, extractable archives for the e2e fixture server, without
 * adding a new npm dependency for test-only fixtures.
 */

/** @param {string} name @param {Buffer} content @param {boolean} executable */
function tarHeader(name, content, executable) {
  const header = Buffer.alloc(512)
  header.write(name, 0, "utf-8")
  header.write((executable ? "755" : "644").padStart(7, "0") + "\0", 100, "utf-8")
  header.write("0000000\0", 108, "utf-8") // uid
  header.write("0000000\0", 116, "utf-8") // gid
  header.write(content.length.toString(8).padStart(11, "0") + "\0", 124, "utf-8")
  header.write("00000000000\0", 136, "utf-8") // mtime
  header.write("        ", 148, "utf-8") // checksum placeholder
  header.write("0", 156, "utf-8") // typeflag: regular file
  header.write("ustar\0", 257, "utf-8")
  header.write("00", 263, "utf-8")

  let checksum = 0
  for (const byte of header) checksum += byte
  header.write(checksum.toString(8).padStart(6, "0") + "\0 ", 148, "utf-8")

  return header
}

/** @param {{ name: string, content: Buffer, executable?: boolean }[]} files */
function buildTar(files) {
  const parts = []
  for (const file of files) {
    parts.push(tarHeader(file.name, file.content, file.executable ?? false))
    parts.push(file.content)
    const padding = (512 - (file.content.length % 512)) % 512
    if (padding > 0) parts.push(Buffer.alloc(padding))
  }
  parts.push(Buffer.alloc(1024)) // two zero-filled end-of-archive blocks
  return Buffer.concat(parts)
}

/** @param {{ name: string, content: Buffer, executable?: boolean }[]} files */
export function buildTarGz(files) {
  return zlib.gzipSync(buildTar(files))
}

/** @param {{ name: string, content: Buffer }[]} files */
export function buildZip(files) {
  const localParts = []
  const centralParts = []
  let offset = 0

  for (const file of files) {
    const nameBuf = Buffer.from(file.name, "utf-8")
    const crc = zlib.crc32(file.content) >>> 0

    const localHeader = Buffer.alloc(30)
    localHeader.writeUInt32LE(0x04034b50, 0)
    localHeader.writeUInt16LE(20, 4) // version needed
    localHeader.writeUInt16LE(0, 6) // flags
    localHeader.writeUInt16LE(0, 8) // method: stored
    localHeader.writeUInt16LE(0, 10) // mod time
    localHeader.writeUInt16LE(0, 12) // mod date
    localHeader.writeUInt32LE(crc, 14)
    localHeader.writeUInt32LE(file.content.length, 18)
    localHeader.writeUInt32LE(file.content.length, 22)
    localHeader.writeUInt16LE(nameBuf.length, 26)
    localHeader.writeUInt16LE(0, 28)

    localParts.push(localHeader, nameBuf, file.content)

    const centralHeader = Buffer.alloc(46)
    centralHeader.writeUInt32LE(0x02014b50, 0)
    centralHeader.writeUInt16LE(20, 4)
    centralHeader.writeUInt16LE(20, 6)
    centralHeader.writeUInt16LE(0, 8)
    centralHeader.writeUInt16LE(0, 10)
    centralHeader.writeUInt16LE(0, 12)
    centralHeader.writeUInt16LE(0, 14)
    centralHeader.writeUInt32LE(crc, 16)
    centralHeader.writeUInt32LE(file.content.length, 20)
    centralHeader.writeUInt32LE(file.content.length, 24)
    centralHeader.writeUInt16LE(nameBuf.length, 28)
    centralHeader.writeUInt16LE(0, 30)
    centralHeader.writeUInt16LE(0, 32)
    centralHeader.writeUInt16LE(0, 34)
    centralHeader.writeUInt16LE(0, 36)
    centralHeader.writeUInt32LE(0, 38)
    centralHeader.writeUInt32LE(offset, 42)

    centralParts.push(centralHeader, nameBuf)

    offset += localHeader.length + nameBuf.length + file.content.length
  }

  const centralDirectory = Buffer.concat(centralParts)
  const localSection = Buffer.concat(localParts)

  const end = Buffer.alloc(22)
  end.writeUInt32LE(0x06054b50, 0)
  end.writeUInt16LE(0, 4)
  end.writeUInt16LE(0, 6)
  end.writeUInt16LE(files.length, 8)
  end.writeUInt16LE(files.length, 10)
  end.writeUInt32LE(centralDirectory.length, 12)
  end.writeUInt32LE(localSection.length, 16)
  end.writeUInt16LE(0, 20)

  return Buffer.concat([localSection, centralDirectory, end])
}
