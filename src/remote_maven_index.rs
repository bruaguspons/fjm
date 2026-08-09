//! Maven Central's `maven-metadata.xml`-based remote version index.
//!
//! Unlike Adoptium, Maven Central has no os/arch axis (the `apache-maven`
//! distribution archive is platform-independent — it only bundles shell
//! scripts and jars) and no embedded checksum in its metadata: checksums are
//! published as a `<artifact>.sha512` sidecar file next to each artifact.

use crate::downloader::ChecksumSource;
use crate::remote_version_index::{RemoteVersionIndex, ResolvedAsset};
use crate::tool_kind::ToolKind;
use crate::version::Version;
use serde::Deserialize;
use url::Url;

const GROUP_PATH: &str = "org/apache/maven";
const ARTIFACT_ID: &str = "apache-maven";

#[derive(Deserialize, Debug)]
struct Metadata {
    versioning: Versioning,
}

#[derive(Deserialize, Debug)]
struct Versioning {
    versions: Versions,
}

#[derive(Deserialize, Debug)]
struct Versions {
    #[serde(rename = "version", default)]
    version: Vec<String>,
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum Error {
    #[error("can't get remote Maven index: {0}")]
    #[diagnostic(transparent)]
    Http(#[from] crate::http::Error),
    #[error("can't decode remote Maven index: {0}")]
    Decode(String),
    #[error("no Maven asset found for version {version}")]
    AssetNotFound { version: String },
    #[error("can't parse the resolved asset's download URL: {0}")]
    InvalidAssetUrl(#[from] url::ParseError),
}

fn metadata_url(base_url: &Url) -> Url {
    Url::parse(&format!(
        "{}/{GROUP_PATH}/{ARTIFACT_ID}/maven-metadata.xml",
        base_url.as_str().trim_end_matches('/')
    ))
    .expect("Maven Central paths always form a valid URL")
}

fn get_body(url: Url) -> Result<String, Error> {
    let response = crate::http::get(url)?;
    response.text().map_err(|source| Error::Http(source.into()))
}

fn decode_metadata(body: &str) -> Result<Metadata, Error> {
    quick_xml::de::from_str(body).map_err(|e| Error::Decode(e.to_string()))
}

/// Lists every Maven version published in Maven Central's
/// `maven-metadata.xml` for `org.apache.maven:apache-maven`.
pub fn list(base_url: &Url) -> Result<Vec<Version>, Error> {
    let body = get_body(metadata_url(base_url))?;
    let metadata = decode_metadata(&body)?;

    Ok(metadata
        .versioning
        .versions
        .version
        .into_iter()
        .filter_map(|v| Version::parse(v, ToolKind::Maven).ok())
        .collect())
}

/// The archive filename used for a given Maven version on this platform.
#[cfg(unix)]
fn archive_name(version: &str) -> String {
    format!("{ARTIFACT_ID}-{version}-bin.tar.gz")
}

#[cfg(windows)]
fn archive_name(version: &str) -> String {
    format!("{ARTIFACT_ID}-{version}-bin.zip")
}

/// Resolves `version` into a downloadable Maven distribution asset.
///
/// Unlike Adoptium's asset resolution, this doesn't need to query an index
/// first: Maven Central's artifact layout is deterministic
/// (`{base}/{group}/{artifact}/{version}/{artifact}-{version}-bin.{ext}`), so
/// the download URL and its `.sha512` sidecar can be constructed directly
/// from the requested version string.
pub fn resolve_asset(base_url: &Url, version_str: &str) -> Result<ResolvedAsset, Error> {
    let version =
        Version::parse(version_str, ToolKind::Maven).map_err(|_| Error::AssetNotFound {
            version: version_str.to_string(),
        })?;

    let name = archive_name(version_str);
    let base = base_url.as_str().trim_end_matches('/');
    let link = Url::parse(&format!(
        "{base}/{GROUP_PATH}/{ARTIFACT_ID}/{version_str}/{name}"
    ))?;
    let sidecar = Url::parse(&format!(
        "{base}/{GROUP_PATH}/{ARTIFACT_ID}/{version_str}/{name}.sha512"
    ))?;

    Ok(ResolvedAsset {
        version,
        link,
        checksum: ChecksumSource::Sidecar(sidecar),
        name,
    })
}

/// Marker implementing [`RemoteVersionIndex`] for Maven Central.
#[allow(
    dead_code,
    reason = "Documents the shared RemoteVersionIndex contract; not yet constructed by any CLI dispatch path, see remote_version_index.rs"
)]
pub struct MavenIndex;

impl RemoteVersionIndex for MavenIndex {
    type Error = Error;

    fn list_remote(&self, config: &crate::config::FjmConfig) -> Result<Vec<Version>, Self::Error> {
        list(config.dist_mirror_for(ToolKind::Maven))
    }

    fn resolve_asset(
        &self,
        config: &crate::config::FjmConfig,
        requested: &Version,
    ) -> Result<ResolvedAsset, Self::Error> {
        resolve_asset(config.dist_mirror_for(ToolKind::Maven), &requested.v_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Shape captured from
    // https://repo.maven.apache.org/maven2/org/apache/maven/apache-maven/maven-metadata.xml
    const METADATA_FIXTURE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<metadata>
  <groupId>org.apache.maven</groupId>
  <artifactId>apache-maven</artifactId>
  <versioning>
    <latest>3.9.9</latest>
    <release>3.9.9</release>
    <versions>
      <version>3.0</version>
      <version>3.8.8</version>
      <version>3.9.8</version>
      <version>3.9.9</version>
    </versions>
    <lastUpdated>20241030090000</lastUpdated>
  </versioning>
</metadata>"#;

    #[test]
    fn test_decode_metadata_lists_every_version() {
        let metadata = decode_metadata(METADATA_FIXTURE).unwrap();
        assert_eq!(
            metadata.versioning.versions.version,
            vec!["3.0", "3.8.8", "3.9.8", "3.9.9"]
        );
    }

    #[test]
    fn test_decode_malformed_xml_is_a_decode_error() {
        assert!(decode_metadata("<not valid xml").is_err());
    }

    #[test]
    fn test_metadata_url_targets_the_right_path() {
        let base = Url::parse("https://repo.maven.apache.org/maven2").unwrap();
        assert_eq!(
            metadata_url(&base).as_str(),
            "https://repo.maven.apache.org/maven2/org/apache/maven/apache-maven/maven-metadata.xml"
        );
    }

    #[test]
    fn test_resolve_asset_builds_link_and_sidecar() {
        let base = Url::parse("https://repo.maven.apache.org/maven2").unwrap();
        let asset = resolve_asset(&base, "3.9.9").unwrap();
        assert!(
            asset
                .link
                .as_str()
                .ends_with("/org/apache/maven/apache-maven/3.9.9/apache-maven-3.9.9-bin.tar.gz")
                || asset
                    .link
                    .as_str()
                    .ends_with("/org/apache/maven/apache-maven/3.9.9/apache-maven-3.9.9-bin.zip")
        );
        assert!(matches!(asset.checksum, ChecksumSource::Sidecar(_)));
    }

    #[test]
    fn test_resolve_asset_rejects_unparseable_version() {
        let base = Url::parse("https://repo.maven.apache.org/maven2").unwrap();
        // Starts with a digit (so it isn't treated as an alias) but isn't a
        // valid semver string, so `Version::parse` hard-errors.
        assert!(resolve_asset(&base, "9.$$$").is_err());
    }
}
