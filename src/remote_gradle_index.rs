//! `services.gradle.org`'s JSON `/versions/all` remote version index.
//!
//! Unlike Maven Central, Gradle's distribution service has no os/arch axis
//! (the `gradle` distribution archive is platform-independent — it only
//! bundles shell scripts and jars, same as Maven) and embeds a sha256
//! checksum directly in each release record (`checksum`) alongside a
//! `checksumUrl` sidecar, so the embedded digest is used directly instead of
//! an extra HTTP round-trip.
//!
//! `services.gradle.org/versions/all` returns every build ever published
//! (snapshots, nightlies, release candidates, milestones, broken builds), so
//! `list`/`resolve_asset` filter down to actual final releases:
//! `snapshot == false`, `rcFor` empty, and `broken == false`.

use crate::downloader::ChecksumSource;
use crate::remote_version_index::{RemoteVersionIndex, ResolvedAsset};
use crate::tool_kind::ToolKind;
use crate::version::Version;
use serde::Deserialize;
use url::Url;

#[derive(Deserialize, Debug)]
struct GradleRelease {
    version: String,
    #[serde(rename = "downloadUrl")]
    download_url: String,
    #[serde(rename = "checksumUrl")]
    checksum_url: Option<String>,
    checksum: Option<String>,
    #[serde(default)]
    snapshot: bool,
    #[serde(default)]
    broken: bool,
    #[serde(rename = "rcFor", default)]
    rc_for: String,
}

impl GradleRelease {
    /// Whether this record is an actual final release, not a snapshot,
    /// nightly, release candidate, or broken build.
    fn is_final_release(&self) -> bool {
        !self.snapshot && self.rc_for.is_empty() && !self.broken
    }
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum Error {
    #[error("can't get remote Gradle index: {0}")]
    #[diagnostic(transparent)]
    Http(#[from] crate::http::Error),
    #[error("can't decode remote Gradle index: {0}")]
    Decode(String),
    #[error("no Gradle asset found for version {version}")]
    AssetNotFound { version: String },
    #[error("Gradle release {version} has no embedded checksum and no checksum URL")]
    NoChecksumAvailable { version: String },
    #[error("can't parse the resolved asset's download URL: {0}")]
    InvalidAssetUrl(#[from] url::ParseError),
}

fn versions_all_url(base_url: &Url) -> Url {
    Url::parse(&format!(
        "{}/versions/all",
        base_url.as_str().trim_end_matches('/')
    ))
    .expect("Gradle distribution service paths always form a valid URL")
}

fn get_body(url: Url) -> Result<String, Error> {
    let response = crate::http::get(url)?;
    response.text().map_err(|source| Error::Http(source.into()))
}

fn decode_releases(body: &str) -> Result<Vec<GradleRelease>, Error> {
    serde_json::from_str(body).map_err(|e| Error::Decode(e.to_string()))
}

fn final_releases(base_url: &Url) -> Result<Vec<GradleRelease>, Error> {
    let body = get_body(versions_all_url(base_url))?;
    let releases = decode_releases(&body)?;
    Ok(releases
        .into_iter()
        .filter(GradleRelease::is_final_release)
        .collect())
}

/// Lists every final Gradle version published in
/// `services.gradle.org/versions/all`, coercing each two-component version
/// string (e.g. `8.10`) into a full three-component semver (e.g. `8.10.0`).
pub fn list(base_url: &Url) -> Result<Vec<Version>, Error> {
    Ok(final_releases(base_url)?
        .into_iter()
        .filter_map(|release| Version::parse(release.version, ToolKind::Gradle).ok())
        .collect())
}

fn checksum_for(release: &GradleRelease) -> Result<ChecksumSource, Error> {
    if let Some(checksum) = &release.checksum {
        return Ok(ChecksumSource::Embedded(checksum.clone()));
    }
    let Some(checksum_url) = &release.checksum_url else {
        return Err(Error::NoChecksumAvailable {
            version: release.version.clone(),
        });
    };
    let url = Url::parse(checksum_url)?;
    let body = get_body(url)?;
    let digest = body
        .split_whitespace()
        .next()
        .ok_or_else(|| Error::NoChecksumAvailable {
            version: release.version.clone(),
        })?
        .to_string();
    Ok(ChecksumSource::Embedded(digest))
}

/// Resolves `version_str` (already coerced, e.g. `8.10.0`) into a
/// downloadable Gradle distribution asset.
///
/// Unlike Maven Central's deterministic artifact layout, Gradle's download
/// URL isn't predictable from the version string alone, so this lists the
/// full index and matches the record whose coerced [`Version`] equals the
/// requested one; the matched record's own `downloadUrl` is authoritative.
pub fn resolve_asset(base_url: &Url, version_str: &str) -> Result<ResolvedAsset, Error> {
    let requested =
        Version::parse(version_str, ToolKind::Gradle).map_err(|_| Error::AssetNotFound {
            version: version_str.to_string(),
        })?;

    let release = final_releases(base_url)?
        .into_iter()
        .find(|release| {
            Version::parse(&release.version, ToolKind::Gradle).is_ok_and(|v| v == requested)
        })
        .ok_or_else(|| Error::AssetNotFound {
            version: version_str.to_string(),
        })?;

    let link = Url::parse(&release.download_url)?;
    let name = link
        .path_segments()
        .and_then(std::iter::Iterator::last)
        .unwrap_or(&release.download_url)
        .to_string();
    let checksum = checksum_for(&release)?;

    Ok(ResolvedAsset {
        version: requested,
        link,
        checksum,
        name,
    })
}

/// Marker implementing [`RemoteVersionIndex`] for Gradle.
#[allow(
    dead_code,
    reason = "Documents the shared RemoteVersionIndex contract; not yet constructed by any CLI dispatch path, see remote_version_index.rs"
)]
pub struct GradleIndex;

impl RemoteVersionIndex for GradleIndex {
    type Error = Error;

    fn list_remote(&self, config: &crate::config::FjmConfig) -> Result<Vec<Version>, Self::Error> {
        list(config.dist_mirror_for(ToolKind::Gradle))
    }

    fn resolve_asset(
        &self,
        config: &crate::config::FjmConfig,
        requested: &Version,
    ) -> Result<ResolvedAsset, Self::Error> {
        resolve_asset(config.dist_mirror_for(ToolKind::Gradle), &requested.v_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Shape captured from https://services.gradle.org/versions/all
    const VERSIONS_FIXTURE: &str = r#"[
      {
        "version": "9.9.0-20260913001956+0000",
        "snapshot": true,
        "nightly": true,
        "rcFor": "",
        "milestoneFor": "",
        "broken": false,
        "downloadUrl": "https://services.gradle.org/distributions-snapshots/gradle-9.9.0-20260913001956+0000-bin.zip",
        "checksumUrl": "https://services.gradle.org/distributions-snapshots/gradle-9.9.0-20260913001956+0000-bin.zip.sha256",
        "checksum": "53bde2d33afcaa50ab62ffc847799807e704979e692a7fe3f6de771737efc6da"
      },
      {
        "version": "9.8.0-rc-1",
        "snapshot": false,
        "nightly": false,
        "rcFor": "9.8.0",
        "milestoneFor": "",
        "broken": false,
        "downloadUrl": "https://services.gradle.org/distributions/gradle-9.8.0-rc-1-bin.zip",
        "checksumUrl": "https://services.gradle.org/distributions/gradle-9.8.0-rc-1-bin.zip.sha256",
        "checksum": "deadbeef"
      },
      {
        "version": "9.0.0",
        "snapshot": false,
        "nightly": false,
        "rcFor": "",
        "milestoneFor": "",
        "broken": false,
        "downloadUrl": "https://services.gradle.org/distributions/gradle-9.0.0-bin.zip",
        "checksumUrl": "https://services.gradle.org/distributions/gradle-9.0.0-bin.zip.sha256",
        "checksum": "acd53f1edaf02f1a8ff99879f8a34b302661a057d9b063ae9e35b552f804d20a"
      },
      {
        "version": "8.10",
        "snapshot": false,
        "nightly": false,
        "rcFor": "",
        "milestoneFor": "",
        "broken": false,
        "downloadUrl": "https://services.gradle.org/distributions/gradle-8.10-bin.zip",
        "checksumUrl": "https://services.gradle.org/distributions/gradle-8.10-bin.zip.sha256",
        "checksum": "84fbba45c7f4c64abc77460e1c00f541e9f960e3c7ed2538f1ede19eacd873ae"
      },
      {
        "version": "8.9-broken",
        "snapshot": false,
        "nightly": false,
        "rcFor": "",
        "milestoneFor": "",
        "broken": true,
        "downloadUrl": "https://services.gradle.org/distributions/gradle-8.9-broken-bin.zip",
        "checksumUrl": "https://services.gradle.org/distributions/gradle-8.9-broken-bin.zip.sha256",
        "checksum": "0000"
      }
    ]"#;

    #[test]
    fn test_decode_releases_lists_every_record() {
        let releases = decode_releases(VERSIONS_FIXTURE).unwrap();
        assert_eq!(releases.len(), 5);
    }

    #[test]
    fn test_decode_malformed_json_is_a_decode_error() {
        assert!(decode_releases("not valid json").is_err());
    }

    #[test]
    fn test_is_final_release_drops_snapshot_rc_and_broken() {
        let releases = decode_releases(VERSIONS_FIXTURE).unwrap();
        let finals: Vec<&str> = releases
            .iter()
            .filter(|r| r.is_final_release())
            .map(|r| r.version.as_str())
            .collect();
        assert_eq!(finals, vec!["9.0.0", "8.10"]);
    }

    #[test]
    fn test_versions_all_url_targets_the_right_path() {
        let base = Url::parse("https://services.gradle.org").unwrap();
        assert_eq!(
            versions_all_url(&base).as_str(),
            "https://services.gradle.org/versions/all"
        );
    }

    #[test]
    fn test_resolve_asset_picks_the_right_download_url_and_coerces_version() {
        // We can't hit the network from a unit test, so exercise the pure
        // matching logic directly against the decoded fixture instead of
        // `resolve_asset` (which fetches over HTTP).
        let releases: Vec<GradleRelease> = decode_releases(VERSIONS_FIXTURE)
            .unwrap()
            .into_iter()
            .filter(GradleRelease::is_final_release)
            .collect();

        let requested = Version::parse("8.10", ToolKind::Gradle).unwrap();
        let matched = releases
            .iter()
            .find(|r| Version::parse(&r.version, ToolKind::Gradle).unwrap() == requested)
            .unwrap();

        assert_eq!(
            matched.download_url,
            "https://services.gradle.org/distributions/gradle-8.10-bin.zip"
        );
    }

    #[test]
    fn test_checksum_for_prefers_embedded_checksum() {
        let releases = decode_releases(VERSIONS_FIXTURE).unwrap();
        let release = releases.iter().find(|r| r.version == "8.10").unwrap();
        let checksum = checksum_for(release).unwrap();
        assert!(
            matches!(checksum, ChecksumSource::Embedded(digest) if digest == release.checksum.clone().unwrap())
        );
    }
}
