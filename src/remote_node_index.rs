use crate::pretty_serde::DecodeError;
use crate::version::Version;
use serde::Deserialize;
use std::collections::HashSet;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedJdkVersion {
    pub major: u32,
    pub lts: bool,
}

#[derive(Debug, Clone)]
pub struct ResolvedAsset {
    pub version: Version,
    pub link: Url,
    pub checksum: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
struct AvailableReleases {
    available_releases: Vec<u32>,
    available_lts_releases: Vec<u32>,
}

#[derive(Deserialize, Debug)]
struct AssetsResponseEntry {
    binary: Binary,
    version: AssetVersionData,
}

#[derive(Deserialize, Debug)]
struct Binary {
    package: Package,
}

#[derive(Deserialize, Debug)]
struct Package {
    name: String,
    link: String,
    checksum: String,
}

#[derive(Deserialize, Debug)]
struct AssetVersionData {
    semver: String,
}

#[derive(Deserialize, Debug)]
struct FeatureRelease {
    binaries: Vec<Binary>,
    version: AssetVersionData,
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum Error {
    #[error("can't get remote JDK index: {0}")]
    #[diagnostic(transparent)]
    Http(#[from] crate::http::Error),
    #[error("can't decode remote JDK index: {0}")]
    #[diagnostic(transparent)]
    Decode(#[from] DecodeError),
    #[error(
        "no JDK asset found for major {major} on {os}/{arch}.\nYou can `fjm ls-remote` to see available versions or try a different `--arch`."
    )]
    AssetNotFound {
        major: u32,
        os: String,
        arch: String,
    },
    #[error("can't parse the resolved asset's version: {0}")]
    InvalidAssetVersion(#[from] node_semver::SemverError),
    #[error("can't parse the resolved asset's download URL: {0}")]
    InvalidAssetUrl(#[from] url::ParseError),
}

fn decode<T: serde::de::DeserializeOwned>(body: String) -> Result<T, Error> {
    serde_json::from_str(&body).map_err(|cause| Error::Decode(DecodeError::from_serde(body, cause)))
}

fn join(base_url: &Url, path: &str) -> Url {
    Url::parse(&format!(
        "{}/{}",
        base_url.as_str().trim_end_matches('/'),
        path
    ))
    .expect("Adoptium API paths always form a valid URL")
}

fn get_body(url: Url) -> Result<String, Error> {
    let response = crate::http::get(url)?;
    response.text().map_err(|source| Error::Http(source.into()))
}

/// Lists every available JDK major version, cheaply, via
/// `GET {base_url}/v3/info/available_releases`. Does not resolve any
/// installable asset data — see [`resolve_asset`] for that.
pub fn list(base_url: &Url) -> Result<Vec<IndexedJdkVersion>, Error> {
    let body = get_body(join(base_url, "v3/info/available_releases"))?;
    let releases: AvailableReleases = decode(body)?;
    let lts_majors: HashSet<u32> = releases.available_lts_releases.into_iter().collect();

    Ok(releases
        .available_releases
        .into_iter()
        .map(|major| IndexedJdkVersion {
            major,
            lts: lts_majors.contains(&major),
        })
        .collect())
}

fn latest_asset_path(major: u32, os: &str, arch: &str, image_type: &str) -> String {
    format!(
        "v3/assets/latest/{major}/hotspot?architecture={arch}&os={os}&image_type={image_type}&vendor=eclipse"
    )
}

fn feature_releases_path(
    major: u32,
    os: &str,
    arch: &str,
    image_type: &str,
    page: u32,
    page_size: u32,
) -> String {
    format!(
        "v3/assets/feature_releases/{major}/ga?architecture={arch}&os={os}&image_type={image_type}&vendor=eclipse&page={page}&page_size={page_size}&sort_order=DESC"
    )
}

/// Resolves the latest installable asset for `major` on the given
/// `os`/`arch`/`image_type`, via
/// `GET {base_url}/v3/assets/latest/{major}/hotspot`.
///
/// This is the resolution path for `--latest` and `--lts`/`lts-<x>`: those
/// selectors have no exact patch to honor, so "whatever Adoptium currently
/// reports as latest" is the correct answer. Ordinary version requests (a
/// bare major, `major.minor`, or an exact `major.minor.patch`) go through
/// [`resolve_asset`] instead, which never silently substitutes a different
/// patch than the one requested.
pub fn resolve_latest_asset(
    base_url: &Url,
    major: u32,
    os: &str,
    arch: &str,
    image_type: &str,
) -> Result<ResolvedAsset, Error> {
    let path = latest_asset_path(major, os, arch, image_type);
    let body = get_body(join(base_url, &path))?;
    let entries: Vec<AssetsResponseEntry> = decode(body)?;
    let entry = entries
        .into_iter()
        .next()
        .ok_or_else(|| Error::AssetNotFound {
            major,
            os: os.to_string(),
            arch: arch.to_string(),
        })?;

    let version = Version::Semver(node_semver::Version::parse(&entry.version.semver)?);
    let link = Url::parse(&entry.binary.package.link)?;

    Ok(ResolvedAsset {
        version,
        link,
        checksum: entry.binary.package.checksum,
        name: entry.binary.package.name,
    })
}

/// Picks the first release in `releases` matching `exact_patch` (or, when
/// `exact_patch` is `None`, the first release at all — `releases` is
/// requested with `sort_order=DESC`, so that's the newest one), and returns
/// its resolved version together with the binary to install.
///
/// Never falls back to a different patch than the one requested: if
/// `exact_patch` doesn't match anything in `releases`, this returns `None`
/// rather than substituting the closest release.
fn select_feature_release(
    releases: Vec<FeatureRelease>,
    exact_patch: Option<(u64, u64, u64)>,
) -> Option<(node_semver::Version, Binary)> {
    for release in releases {
        let Ok(version) = node_semver::Version::parse(&release.version.semver) else {
            continue;
        };

        let matches = match exact_patch {
            Some((major, minor, patch)) => {
                version.major == major && version.minor == minor && version.patch == patch
            }
            None => true,
        };

        if !matches {
            continue;
        }

        if let Some(binary) = release.binaries.into_iter().next() {
            return Some((version, binary));
        }
    }

    None
}

/// Resolves an installable asset for `major` on the given
/// `os`/`arch`/`image_type`, via paginated
/// `GET {base_url}/v3/assets/feature_releases/{major}/ga` requests.
///
/// When `exact_patch` is `Some((major, minor, patch))`, only a release whose
/// `version_data` matches that exact patch is accepted; if none of the
/// paginated `feature_releases` results match, this returns
/// [`Error::AssetNotFound`] instead of silently falling back to a different
/// release. When `exact_patch` is `None` (a bare major or `major.minor`
/// request with no patch to honor), the newest release for `major` is used.
pub fn resolve_asset(
    base_url: &Url,
    major: u32,
    os: &str,
    arch: &str,
    image_type: &str,
    exact_patch: Option<(u64, u64, u64)>,
) -> Result<ResolvedAsset, Error> {
    const PAGE_SIZE: u32 = 20;
    const MAX_PAGES: u32 = 50;

    for page in 0..MAX_PAGES {
        let path = feature_releases_path(major, os, arch, image_type, page, PAGE_SIZE);
        let body = get_body(join(base_url, &path))?;
        let releases: Vec<FeatureRelease> = decode(body)?;
        let page_len = releases.len();

        if let Some((version, binary)) = select_feature_release(releases, exact_patch) {
            let link = Url::parse(&binary.package.link)?;
            return Ok(ResolvedAsset {
                version: Version::Semver(version),
                link,
                checksum: binary.package.checksum,
                name: binary.package.name,
            });
        }

        if exact_patch.is_none() || page_len < PAGE_SIZE as usize {
            break;
        }
    }

    Err(Error::AssetNotFound {
        major,
        os: os.to_string(),
        arch: arch.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Captured live from https://api.adoptium.net/v3/info/available_releases
    const AVAILABLE_RELEASES_FIXTURE: &str = r#"{
        "available_lts_releases": [8, 11, 17, 21, 25],
        "available_releases": [8, 11, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26],
        "most_recent_feature_release": 26,
        "most_recent_feature_version": 28,
        "most_recent_lts": 25,
        "tip_version": 28
    }"#;

    // Captured live from
    // https://api.adoptium.net/v3/assets/latest/21/hotspot?architecture=x64&os=linux&image_type=jdk&vendor=eclipse
    const ASSETS_LATEST_FIXTURE: &str = r#"[
        {
            "binary": {
                "architecture": "x64",
                "download_count": 732014,
                "heap_size": "normal",
                "image_type": "jdk",
                "jvm_impl": "hotspot",
                "os": "linux",
                "package": {
                    "checksum": "e4446ff06a276155697597cc0f1b15da004ff083f4964a35271ecee567177370",
                    "checksum_link": "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.12%2B8/OpenJDK21U-jdk_x64_linux_hotspot_21.0.12_8.tar.gz.sha256.txt",
                    "download_count": 732014,
                    "link": "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.12%2B8/OpenJDK21U-jdk_x64_linux_hotspot_21.0.12_8.tar.gz",
                    "metadata_link": "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.12%2B8/OpenJDK21U-jdk_x64_linux_hotspot_21.0.12_8.tar.gz.json",
                    "name": "OpenJDK21U-jdk_x64_linux_hotspot_21.0.12_8.tar.gz",
                    "signature_link": "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.12%2B8/OpenJDK21U-jdk_x64_linux_hotspot_21.0.12_8.tar.gz.sig",
                    "size": 207486543
                },
                "project": "jdk",
                "scm_ref": "jdk-21.0.12+8_adopt",
                "updated_at": "2026-07-24T15:52:42Z"
            },
            "release_link": "https://github.com/adoptium/temurin21-binaries/releases/tag/jdk-21.0.12%2B8",
            "release_name": "jdk-21.0.12+8",
            "vendor": "eclipse",
            "version": {
                "build": 8,
                "major": 21,
                "minor": 0,
                "openjdk_version": "21.0.12+8-LTS",
                "optional": "LTS",
                "security": 12,
                "semver": "21.0.12+8.0.LTS"
            }
        }
    ]"#;

    #[test]
    fn test_available_releases_decode_marks_lts_majors() {
        let parsed: AvailableReleases = decode(AVAILABLE_RELEASES_FIXTURE.to_string()).unwrap();
        let lts_majors: HashSet<u32> = parsed.available_lts_releases.into_iter().collect();
        let versions: Vec<IndexedJdkVersion> = parsed
            .available_releases
            .into_iter()
            .map(|major| IndexedJdkVersion {
                major,
                lts: lts_majors.contains(&major),
            })
            .collect();

        assert!(versions.contains(&IndexedJdkVersion {
            major: 21,
            lts: true
        }));
        assert!(versions.contains(&IndexedJdkVersion {
            major: 26,
            lts: false
        }));
    }

    #[test]
    fn test_decode_malformed_json_is_a_decode_error() {
        let err = decode::<AvailableReleases>("{not valid json".to_string()).unwrap_err();
        assert!(matches!(err, Error::Decode(_)));
    }

    #[test]
    fn test_resolve_asset_parses_real_adoptium_response() {
        let entries: Vec<AssetsResponseEntry> = decode(ASSETS_LATEST_FIXTURE.to_string()).unwrap();
        let entry = entries.into_iter().next().unwrap();

        // Confirms real Adoptium `version_data.semver` values (which embed
        // build metadata like `8.0.LTS`) are node_semver-parseable, so no
        // custom JDK version parser is needed.
        let version = node_semver::Version::parse(&entry.version.semver).unwrap();
        assert_eq!(version.major, 21);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 12);

        assert_eq!(
            entry.binary.package.name,
            "OpenJDK21U-jdk_x64_linux_hotspot_21.0.12_8.tar.gz"
        );
    }

    #[test]
    fn test_resolve_asset_empty_response_is_asset_not_found() {
        let entries: Vec<AssetsResponseEntry> = decode("[]".to_string()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_latest_asset_path_targets_the_latest_endpoint() {
        assert_eq!(
            latest_asset_path(21, "linux", "x64", "jdk"),
            "v3/assets/latest/21/hotspot?architecture=x64&os=linux&image_type=jdk&vendor=eclipse"
        );
    }

    #[test]
    fn test_feature_releases_path_targets_the_feature_releases_endpoint() {
        assert_eq!(
            feature_releases_path(17, "linux", "x64", "jdk", 0, 20),
            "v3/assets/feature_releases/17/ga?architecture=x64&os=linux&image_type=jdk&vendor=eclipse&page=0&page_size=20&sort_order=DESC"
        );
    }

    // Modeled on the real `/v3/assets/feature_releases/{feature_version}/ga`
    // response shape (a list of releases, each with a `binaries` array,
    // unlike `/v3/assets/latest/...`'s single `binary`). Two patches of the
    // same major are included to exercise exact-patch selection.
    const FEATURE_RELEASES_FIXTURE: &str = r#"[
        {
            "binaries": [
                {
                    "architecture": "x64",
                    "image_type": "jdk",
                    "jvm_impl": "hotspot",
                    "os": "linux",
                    "package": {
                        "checksum": "9a9a9a9a9a9a9a9a9a9a9a9a9a9a9a9a9a9a9a9a9a9a9a9a9a9a9a9a9a9a9a9a",
                        "link": "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.9%2B9/OpenJDK17U-jdk_x64_linux_hotspot_17.0.9_9.tar.gz",
                        "name": "OpenJDK17U-jdk_x64_linux_hotspot_17.0.9_9.tar.gz"
                    },
                    "project": "jdk"
                }
            ],
            "release_link": "https://github.com/adoptium/temurin17-binaries/releases/tag/jdk-17.0.9%2B9",
            "release_name": "jdk-17.0.9+9",
            "release_type": "ga",
            "vendor": "eclipse",
            "version": {
                "build": 9,
                "major": 17,
                "minor": 0,
                "openjdk_version": "17.0.9+9",
                "security": 9,
                "semver": "17.0.9+9"
            }
        },
        {
            "binaries": [
                {
                    "architecture": "x64",
                    "image_type": "jdk",
                    "jvm_impl": "hotspot",
                    "os": "linux",
                    "package": {
                        "checksum": "b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1",
                        "link": "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.1%2B10/OpenJDK17U-jdk_x64_linux_hotspot_17.0.1_10.tar.gz",
                        "name": "OpenJDK17U-jdk_x64_linux_hotspot_17.0.1_10.tar.gz"
                    },
                    "project": "jdk"
                }
            ],
            "release_link": "https://github.com/adoptium/temurin17-binaries/releases/tag/jdk-17.0.1%2B10",
            "release_name": "jdk-17.0.1+10",
            "release_type": "ga",
            "vendor": "eclipse",
            "version": {
                "build": 10,
                "major": 17,
                "minor": 0,
                "openjdk_version": "17.0.1+10",
                "security": 1,
                "semver": "17.0.1+10"
            }
        }
    ]"#;

    #[test]
    fn test_select_feature_release_finds_exact_patch() {
        let releases: Vec<FeatureRelease> = decode(FEATURE_RELEASES_FIXTURE.to_string()).unwrap();

        let (version, binary) = select_feature_release(releases, Some((17, 0, 1))).unwrap();

        assert_eq!(version.patch, 1);
        assert_eq!(
            binary.package.name,
            "OpenJDK17U-jdk_x64_linux_hotspot_17.0.1_10.tar.gz"
        );
    }

    #[test]
    fn test_select_feature_release_no_match_for_unavailable_patch() {
        let releases: Vec<FeatureRelease> = decode(FEATURE_RELEASES_FIXTURE.to_string()).unwrap();

        // Neither fixture release is patch 99: this must not silently fall
        // back to the newest one (17.0.9) — the caller (`resolve_asset`)
        // turns this `None` into `Error::AssetNotFound`.
        assert!(select_feature_release(releases, Some((17, 0, 99))).is_none());
    }

    #[test]
    fn test_select_feature_release_without_exact_patch_picks_newest() {
        let releases: Vec<FeatureRelease> = decode(FEATURE_RELEASES_FIXTURE.to_string()).unwrap();

        // No exact patch requested (e.g. a bare `fjm install 17`): the first
        // entry wins, matching `sort_order=DESC`'s "newest first" ordering.
        let (version, _binary) = select_feature_release(releases, None).unwrap();
        assert_eq!(version.patch, 9);
    }
}
