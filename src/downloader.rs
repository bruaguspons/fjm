use crate::archive::{Archive, Error as ExtractError};
use crate::directory_portal::DirectoryPortal;
use crate::progress::ResponseProgress;
use crate::remote_version_index::ResolvedAsset;
use crate::version::Version;
use indicatif::ProgressDrawTarget;
use sha2::{Digest, Sha256, Sha512};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use thiserror::Error;
use url::Url;

/// Where an asset's expected checksum comes from.
///
/// Adoptium embeds a sha256 checksum directly in its JSON asset metadata
/// (`Embedded`); Maven Central publishes a separate `<artifact>.sha512`
/// sidecar file next to the artifact instead (`Sidecar`), requiring one
/// extra HTTP round-trip to fetch it.
#[derive(Debug, Clone)]
pub enum ChecksumSource {
    Embedded(String),
    Sidecar(Url),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    HttpError {
        #[from]
        source: crate::http::Error,
    },
    #[error(transparent)]
    IoError {
        #[from]
        source: std::io::Error,
    },
    #[error("Can't extract the file: {}", source)]
    CantExtractFile {
        #[from]
        source: ExtractError,
    },
    #[error("checksum mismatch for {version}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        version: Version,
        expected: String,
        actual: String,
    },
    #[error("Version already installed at {:?}", path)]
    VersionAlreadyInstalled { path: PathBuf },
    #[error("Don't know how to extract archive `{name}`")]
    UnsupportedArchiveFormat { name: String },
    #[error("can't fetch checksum sidecar {url}: {source}")]
    ChecksumSidecarFetch {
        url: Url,
        source: crate::http::Error,
    },
}

#[cfg(unix)]
fn archive_for_name(name: &str) -> Result<Archive, Error> {
    if name.ends_with(".tar.gz") {
        Ok(Archive::TarGz)
    } else if name.ends_with(".tar.xz") {
        Ok(Archive::TarXz)
    } else {
        Err(Error::UnsupportedArchiveFormat {
            name: name.to_string(),
        })
    }
}

#[cfg(windows)]
fn archive_for_name(name: &str) -> Result<Archive, Error> {
    if name.to_ascii_lowercase().ends_with(".zip") {
        Ok(Archive::Zip)
    } else {
        Err(Error::UnsupportedArchiveFormat {
            name: name.to_string(),
        })
    }
}

fn verify_checksum(version: &Version, bytes: &[u8], expected: &str) -> Result<(), Error> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let actual = format!("{:x}", hasher.finalize());

    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(Error::ChecksumMismatch {
            version: version.clone(),
            expected: expected.to_string(),
            actual,
        })
    }
}

fn verify_checksum_sha512(version: &Version, bytes: &[u8], expected: &str) -> Result<(), Error> {
    let mut hasher = Sha512::new();
    hasher.update(bytes);
    let actual = format!("{:x}", hasher.finalize());

    // Maven's `.sha512` sidecar files sometimes contain just the hex digest,
    // and sometimes `<digest>  <filename>` (like `sha512sum` output) — only
    // the first whitespace-delimited token is the digest.
    let expected_digest = expected.split_whitespace().next().unwrap_or(expected);

    if actual.eq_ignore_ascii_case(expected_digest) {
        Ok(())
    } else {
        Err(Error::ChecksumMismatch {
            version: version.clone(),
            expected: expected_digest.to_string(),
            actual,
        })
    }
}

fn verify_checksum_source(
    version: &Version,
    bytes: &[u8],
    source: &ChecksumSource,
) -> Result<(), Error> {
    match source {
        ChecksumSource::Embedded(checksum) => verify_checksum(version, bytes, checksum),
        ChecksumSource::Sidecar(url) => {
            let response =
                crate::http::get(url.clone()).map_err(|source| Error::ChecksumSidecarFetch {
                    url: url.clone(),
                    source,
                })?;
            let body = response
                .text()
                .map_err(|source| Error::ChecksumSidecarFetch {
                    url: url.clone(),
                    source: source.into(),
                })?;
            verify_checksum_sha512(version, bytes, body.trim())
        }
    }
}

/// Adoptium archives contain a single top-level directory (e.g.
/// `jdk-21.0.12+8/`). Flatten it so `installation/bin/java` exists directly,
/// matching what every downstream consumer of `installation_path` expects.
fn flatten_single_top_level_dir(dir: &Path) -> std::io::Result<()> {
    let entries: Vec<_> = std::fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    let [root] = entries.as_slice() else {
        return Ok(());
    };
    if !root.file_type()?.is_dir() {
        return Ok(());
    }

    let root_path = root.path();
    for child in std::fs::read_dir(&root_path)? {
        let child = child?;
        std::fs::rename(child.path(), dir.join(child.file_name()))?;
    }
    std::fs::remove_dir(root_path)?;
    Ok(())
}

/// Install a tool distribution (JDK or Maven) from a resolved asset.
///
/// Shared by every `ToolKind`: the download → checksum-verify → extract →
/// flatten → atomically-teleport-into-place pipeline is identical, only the
/// [`ChecksumSource`] and archive format differ per resolved asset.
pub fn install_node_dist<P: AsRef<Path>>(
    resolved_asset: &ResolvedAsset,
    installations_dir: P,
    show_progress: bool,
) -> Result<(), Error> {
    let target = installations_dir
        .as_ref()
        .join(resolved_asset.version.v_str())
        .join("installation");

    if target.exists() {
        return Err(Error::VersionAlreadyInstalled { path: target });
    }

    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let response = crate::http::get(resolved_asset.link.clone())?;
    let mut reader: Box<dyn Read> = if show_progress {
        Box::new(ResponseProgress::new(
            response,
            ProgressDrawTarget::stderr(),
        ))
    } else {
        Box::new(response)
    };

    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    drop(reader);

    verify_checksum_source(&resolved_asset.version, &bytes, &resolved_asset.checksum)?;

    let archive = archive_for_name(&resolved_asset.name)?;
    let portal = DirectoryPortal::new_in(installations_dir, target);
    archive.extract_archive_into(&portal, Cursor::new(bytes))?;
    flatten_single_top_level_dir(&portal)?;
    portal.teleport()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tool_kind::ToolKind;

    #[test]
    fn test_checksum_mismatch_is_detected() {
        let version = Version::parse("21.0.0", ToolKind::Java).unwrap();
        let err = verify_checksum(&version, b"hello world", &"0".repeat(64)).unwrap_err();
        assert!(matches!(err, Error::ChecksumMismatch { .. }));
    }

    #[test]
    fn test_checksum_match_succeeds() {
        let version = Version::parse("21.0.0", ToolKind::Java).unwrap();
        // sha256("hello world")
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert!(verify_checksum(&version, b"hello world", expected).is_ok());
    }

    #[test]
    fn test_checksum_verification_is_case_insensitive() {
        let version = Version::parse("21.0.0", ToolKind::Java).unwrap();
        let expected = "B94D27B9934D3E08A52E52D7DA7DABFAC484EFE37A5380EE9088F7ACE2EFCDE9";
        assert!(verify_checksum(&version, b"hello world", expected).is_ok());
    }

    #[test]
    fn test_sha512_sidecar_checksum_match_succeeds() {
        let version = Version::parse("3.9.9", ToolKind::Maven).unwrap();
        // sha512("hello world")
        let expected = "309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f";
        assert!(verify_checksum_sha512(&version, b"hello world", expected).is_ok());
    }

    #[test]
    fn test_sha512_sidecar_checksum_tolerates_sha512sum_style_suffix() {
        let version = Version::parse("3.9.9", ToolKind::Maven).unwrap();
        let expected = "309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f  apache-maven-3.9.9-bin.tar.gz";
        assert!(verify_checksum_sha512(&version, b"hello world", expected).is_ok());
    }

    #[test]
    fn test_sha512_sidecar_checksum_mismatch_is_detected() {
        let version = Version::parse("3.9.9", ToolKind::Maven).unwrap();
        let err = verify_checksum_sha512(&version, b"hello world", &"0".repeat(128)).unwrap_err();
        assert!(matches!(err, Error::ChecksumMismatch { .. }));
    }

    #[cfg(unix)]
    #[test]
    fn test_archive_for_name_picks_tar_gz() {
        assert!(matches!(
            archive_for_name("OpenJDK21U-jdk_x64_linux_hotspot_21.0.12_8.tar.gz"),
            Ok(Archive::TarGz)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn test_archive_for_name_rejects_unknown_extension() {
        assert!(matches!(
            archive_for_name("weird.rar"),
            Err(Error::UnsupportedArchiveFormat { .. })
        ));
    }

    #[test]
    fn test_flatten_single_top_level_dir() {
        let temp = tempfile::tempdir().unwrap();
        let nested = temp.path().join("jdk-21.0.12+8");
        std::fs::create_dir(&nested).unwrap();
        std::fs::write(nested.join("marker"), b"hi").unwrap();

        flatten_single_top_level_dir(temp.path()).unwrap();

        assert!(temp.path().join("marker").exists());
        assert!(!nested.exists());
    }

    #[test]
    fn test_flatten_is_a_noop_with_multiple_entries() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("a"), b"a").unwrap();
        std::fs::write(temp.path().join("b"), b"b").unwrap();

        flatten_single_top_level_dir(temp.path()).unwrap();

        assert!(temp.path().join("a").exists());
        assert!(temp.path().join("b").exists());
    }
}
