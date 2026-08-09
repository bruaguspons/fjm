use super::command::Command;
use super::r#use::Use;
use crate::alias::create_alias;
use crate::arch::{get_safe_arch, ArchError};
use crate::config::FjmConfig;
use crate::downloader::{install_node_dist, Error as DownloaderError};
use crate::lts::LtsType;
use crate::outln;
use crate::progress::ProgressConfig;
use crate::remote_maven_index;
use crate::remote_node_index;
use crate::remote_version_index::ResolvedAsset;
use crate::tool_kind::ToolKind;
use crate::user_version::UserVersion;
use crate::user_version_reader::UserVersionReader;
use crate::version::Version;
use crate::version_files::get_user_version_for_directory_for_tool;
use colored::Colorize;
use log::debug;
use thiserror::Error;

#[derive(clap::Parser, Debug, Default)]
pub struct Install {
    /// A version string. Can be a partial semver or a LTS version name by the format lts/NAME
    pub version: Option<UserVersion>,

    /// Which tool to install a version for.
    #[clap(long, value_enum, default_value_t)]
    pub tool: ToolKind,

    /// Install latest LTS (Java only)
    #[clap(long, conflicts_with_all = &["version", "latest"])]
    pub lts: bool,

    /// Install latest version
    #[clap(long, conflicts_with_all = &["version", "lts"])]
    pub latest: bool,

    /// Show an interactive progress bar for the download
    /// status.
    #[clap(long, default_value_t)]
    #[arg(value_enum)]
    pub progress: ProgressConfig,

    /// Use the installed version immediately after installation
    #[clap(long)]
    pub r#use: bool,
}

impl Install {
    fn version(self) -> Result<Option<UserVersion>, Error> {
        match self {
            Self {
                version: v,
                lts: false,
                latest: false,
                ..
            } => Ok(v),
            Self {
                version: None,
                lts: true,
                latest: false,
                ..
            } => Ok(Some(UserVersion::Full(Version::Lts(LtsType::Latest)))),
            Self {
                version: None,
                lts: false,
                latest: true,
                ..
            } => Ok(Some(UserVersion::Full(Version::Latest))),
            _ => Err(Error::TooManyVersionsProvided),
        }
    }
}

impl Command for Install {
    type Error = Error;

    fn apply(self, config: &FjmConfig) -> Result<(), Self::Error> {
        match self.tool {
            ToolKind::Java => install_java(self, config),
            ToolKind::Maven => install_maven(self, config),
        }
    }
}

fn install_java(install: Install, config: &FjmConfig) -> Result<(), Error> {
    let current_dir = std::env::current_dir().unwrap();
    let show_progress = install.progress.enabled(config);
    let use_installed = install.r#use;
    let tool = ToolKind::Java;

    let current_version = install
        .version()?
        .or_else(|| get_user_version_for_directory_for_tool(current_dir, config, tool))
        .ok_or(Error::CantInferVersion)?;

    if let UserVersion::Full(v @ (Version::Bypassed | Version::Alias(_))) = &current_version {
        return Err(Error::UninstallableVersion { version: v.clone() });
    }

    let available_versions = remote_node_index::list(config.dist_mirror_for(tool))
        .map_err(|source| Error::CantListRemoteVersions { source })?;

    let major = resolve_major(&current_version, &available_versions)?;

    // Automatically swap Apple Silicon to x64 arch for appropriate versions.
    let probe_version = Version::Semver(
        node_semver::Version::parse(format!("{major}.0.0"))
            .expect("a bare major number is always a valid semver"),
    );
    let safe_arch = get_safe_arch(config.arch, &probe_version);

    let (os, arch) = safe_arch
        .adoptium_os_arch()
        .map_err(|source| Error::UnsupportedArch { source })?;

    let resolved_asset = match asset_resolution_strategy(&current_version) {
        AssetResolutionStrategy::Latest => remote_node_index::resolve_latest_asset(
            config.dist_mirror_for(tool),
            major,
            os,
            arch,
            "jdk",
        )
        .map_err(|source| Error::CantResolveAsset { source })?,
        AssetResolutionStrategy::ExactPatch(major_v, minor_v, patch_v) => {
            remote_node_index::resolve_asset(
                config.dist_mirror_for(tool),
                major,
                os,
                arch,
                "jdk",
                Some((major_v, minor_v, patch_v)),
            )
            .map_err(|source| Error::CantResolveAsset { source })?
        }
        AssetResolutionStrategy::AnyInMajor => remote_node_index::resolve_asset(
            config.dist_mirror_for(tool),
            major,
            os,
            arch,
            "jdk",
            None,
        )
        .map_err(|source| Error::CantResolveAsset { source })?,
    };

    let version = resolved_asset.version.clone();

    let version_str = format!("JDK {version}");
    outln!(
        config,
        Info,
        "Installing {} ({})",
        version_str.cyan(),
        safe_arch.as_str()
    );

    install_and_alias(
        config,
        tool,
        &current_version,
        &version,
        &resolved_asset,
        show_progress,
        use_installed,
    )
}

fn install_maven(install: Install, config: &FjmConfig) -> Result<(), Error> {
    let current_dir = std::env::current_dir().unwrap();
    let show_progress = install.progress.enabled(config);
    let use_installed = install.r#use;
    let tool = ToolKind::Maven;

    let current_version = install
        .version()?
        .or_else(|| get_user_version_for_directory_for_tool(current_dir, config, tool))
        .ok_or(Error::CantInferVersion)?;

    if let UserVersion::Full(v @ (Version::Bypassed | Version::Alias(_))) = &current_version {
        return Err(Error::UninstallableVersion { version: v.clone() });
    }

    let requested_version_str = match &current_version {
        UserVersion::Full(Version::Latest) => {
            let available = remote_maven_index::list(config.dist_mirror_for(tool))
                .map_err(|source| Error::CantListRemoteMavenVersions { source })?;
            let picked = available.into_iter().max().ok_or(Error::CantFindLatest)?;
            picked.v_str()
        }
        UserVersion::Full(Version::Semver(_)) => current_version
            .to_string()
            .trim_start_matches('v')
            .to_string(),
        UserVersion::OnlyMajor(_) | UserVersion::MajorMinor(_, _) => {
            // Maven Central doesn't expose a major-only listing endpoint the
            // way Adoptium does; resolve the exact matching version(s) from
            // the full listing instead.
            let available = remote_maven_index::list(config.dist_mirror_for(tool))
                .map_err(|source| Error::CantListRemoteMavenVersions { source })?;
            current_version
                .to_version_for_tool(&available, config, tool)
                .ok_or_else(|| Error::CantFindMavenVersion {
                    requested_version: current_version.clone(),
                })?
                .v_str()
        }
        UserVersion::Full(Version::Lts(_)) => {
            return Err(Error::LtsNotSupportedForMaven);
        }
        UserVersion::SemverRange(_) | UserVersion::Full(Version::Bypassed | Version::Alias(_)) => {
            return Err(Error::CantFindMavenVersion {
                requested_version: current_version.clone(),
            });
        }
    };

    let resolved_asset =
        remote_maven_index::resolve_asset(config.dist_mirror_for(tool), &requested_version_str)
            .map_err(|source| Error::CantResolveMavenAsset { source })?;

    let version = resolved_asset.version.clone();

    outln!(
        config,
        Info,
        "Installing {}",
        format!("Maven {version}").cyan()
    );

    install_and_alias(
        config,
        tool,
        &current_version,
        &version,
        &resolved_asset,
        show_progress,
        use_installed,
    )
}

fn install_and_alias(
    config: &FjmConfig,
    tool: ToolKind,
    current_version: &UserVersion,
    version: &Version,
    resolved_asset: &ResolvedAsset,
    show_progress: bool,
    use_installed: bool,
) -> Result<(), Error> {
    match install_node_dist(
        resolved_asset,
        config.installations_dir_for(tool),
        show_progress,
    ) {
        Err(err @ DownloaderError::VersionAlreadyInstalled { .. }) => {
            outln!(config, Error, "{} {}", "warning:".bold().yellow(), err);
        }
        Err(source) => Err(Error::DownloadError { source })?,
        Ok(()) => {}
    }

    if !config.default_version_dir_for(tool).exists() {
        debug!("Tagging {} as the default version", version.v_str().cyan());
        create_alias(config, "default", version, tool)?;
    }

    if let Some(tagged_alias) = current_version.inferred_alias() {
        tag_alias(config, version, &tagged_alias, tool)?;
    }

    if use_installed {
        use_installed_version(version, config, tool)?;
    }

    Ok(())
}

/// Resolves the requested version into a concrete installable JDK major.
///
/// Adoptium's listing only exposes majors (no per-patch data), so this only
/// picks the major to fetch assets for — it doesn't preserve the requested
/// minor/patch. Exact-patch requests (e.g. `17.0.1`) are honored downstream,
/// in the choice between [`remote_node_index::resolve_asset`] (exact patch,
/// via `feature_releases`) and [`remote_node_index::resolve_latest_asset`]
/// (`--latest`/`--lts`, via `latest`) in [`Install::apply`].
/// Which asset-resolution endpoint a requested version should go through.
///
/// `--latest`/`--lts` have no exact patch to honor, so they use Adoptium's
/// `latest` endpoint (whatever it currently reports). Every other request
/// goes through the paginated `feature_releases` endpoint instead: an exact
/// `major.minor.patch` request is honored exactly (`ExactPatch`, never
/// silently substituting a different patch), while a bare major or
/// `major.minor` request has no patch to honor and gets the newest release
/// for that major (`AnyInMajor`).
#[derive(Debug, PartialEq, Eq)]
enum AssetResolutionStrategy {
    Latest,
    ExactPatch(u64, u64, u64),
    AnyInMajor,
}

fn asset_resolution_strategy(current_version: &UserVersion) -> AssetResolutionStrategy {
    match current_version {
        UserVersion::Full(Version::Latest | Version::Lts(_)) => AssetResolutionStrategy::Latest,
        UserVersion::Full(Version::Semver(v)) => {
            AssetResolutionStrategy::ExactPatch(v.major, v.minor, v.patch)
        }
        UserVersion::OnlyMajor(_)
        | UserVersion::MajorMinor(_, _)
        | UserVersion::SemverRange(_)
        | UserVersion::Full(Version::Bypassed | Version::Alias(_)) => {
            AssetResolutionStrategy::AnyInMajor
        }
    }
}

fn resolve_major(
    current_version: &UserVersion,
    available_versions: &[remote_node_index::IndexedJdkVersion],
) -> Result<u32, Error> {
    match current_version {
        UserVersion::Full(Version::Semver(actual_version)) => u32::try_from(actual_version.major)
            .map_err(|_| Error::CantFindNodeVersion {
                requested_version: current_version.clone(),
            }),
        UserVersion::Full(Version::Lts(lts_type)) => {
            let picked = lts_type
                .pick_latest(available_versions)
                .map_err(|source| Error::LtsNotSupported {
                    lts_type: lts_type.clone(),
                    source,
                })?
                .ok_or_else(|| Error::CantFindRelevantLts {
                    lts_type: lts_type.clone(),
                })?;
            debug!(
                "Resolved {} into JDK {}",
                Version::Lts(lts_type.clone()).v_str().cyan(),
                picked.major
            );
            Ok(picked.major)
        }
        UserVersion::Full(Version::Latest) => {
            let picked = available_versions
                .iter()
                .max_by_key(|v| v.major)
                .ok_or(Error::CantFindLatest)?;
            debug!(
                "Resolved {} into JDK {}",
                Version::Latest.v_str().cyan(),
                picked.major
            );
            Ok(picked.major)
        }
        UserVersion::OnlyMajor(major) | UserVersion::MajorMinor(major, _) => {
            let major = u32::try_from(*major).map_err(|_| Error::CantFindNodeVersion {
                requested_version: current_version.clone(),
            })?;
            available_versions
                .iter()
                .any(|v| v.major == major)
                .then_some(major)
                .ok_or_else(|| Error::CantFindNodeVersion {
                    requested_version: current_version.clone(),
                })
        }
        UserVersion::SemverRange(_) | UserVersion::Full(Version::Bypassed | Version::Alias(_)) => {
            Err(Error::CantFindNodeVersion {
                requested_version: current_version.clone(),
            })
        }
    }
}

fn tag_alias(
    config: &FjmConfig,
    matched_version: &Version,
    alias: &Version,
    tool: ToolKind,
) -> Result<(), Error> {
    let alias_name = alias.v_str();
    debug!(
        "Tagging {} as alias for {}",
        alias_name.cyan(),
        matched_version.v_str().cyan()
    );
    create_alias(config, &alias_name, matched_version, tool)?;

    Ok(())
}

fn use_installed_version(
    version: &Version,
    config: &FjmConfig,
    tool: ToolKind,
) -> Result<(), Error> {
    Use {
        version: Some(UserVersionReader::Direct(UserVersion::Full(
            version.clone(),
        ))),
        tool,
        lts: false,
        latest: false,
        install_if_missing: false,
        silent_if_unchanged: false,
        info_to_stderr: false,
    }
    .apply(config)
    .map_err(|source| Error::UseError {
        source: Box::new(source),
    })?;
    Ok(())
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Can't download the requested binary: {}", source)]
    DownloadError { source: DownloaderError },
    #[error(transparent)]
    IoError {
        #[from]
        source: std::io::Error,
    },
    #[error(transparent)]
    UseError {
        source: Box<<Use as Command>::Error>,
    },
    #[error("Can't find version in dotfiles. Please provide a version manually to the command.")]
    CantInferVersion,
    #[error(transparent)]
    CantListRemoteVersions { source: remote_node_index::Error },
    #[error(transparent)]
    CantListRemoteMavenVersions { source: remote_maven_index::Error },
    #[error(
        "Can't find a JDK version that matches {} in remote",
        requested_version
    )]
    CantFindNodeVersion { requested_version: UserVersion },
    #[error(
        "Can't find a Maven version that matches {} in remote",
        requested_version
    )]
    CantFindMavenVersion { requested_version: UserVersion },
    #[error("Can't find relevant LTS named {}", lts_type)]
    CantFindRelevantLts { lts_type: crate::lts::LtsType },
    #[error("Can't resolve LTS {}: {}", lts_type, source)]
    LtsNotSupported {
        lts_type: crate::lts::LtsType,
        source: crate::lts::Error,
    },
    #[error("Maven has no LTS concept; pass an exact version or --latest instead")]
    LtsNotSupportedForMaven,
    #[error("Can't find any versions in the upstream version index.")]
    CantFindLatest,
    #[error("The requested version is not installable: {}", version.v_str())]
    UninstallableVersion { version: Version },
    #[error("Too many versions provided. Please don't use --lts with a version string.")]
    TooManyVersionsProvided,
    #[error(transparent)]
    UnsupportedArch { source: ArchError },
    #[error(transparent)]
    CantResolveAsset { source: remote_node_index::Error },
    #[error(transparent)]
    CantResolveMavenAsset { source: remote_maven_index::Error },
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::str::FromStr;

    #[test]
    #[ignore = "hits the real Adoptium API; run manually or against a proxy via the e2e suite"]
    fn test_set_default_on_new_installation() {
        let base_dir = tempfile::tempdir().unwrap();
        let config = FjmConfig::default().with_base_dir(Some(base_dir.path().to_path_buf()));
        assert!(!config.default_version_dir().exists());

        Install {
            version: UserVersion::from_str("17").ok(),
            tool: ToolKind::Java,
            lts: false,
            latest: false,
            progress: ProgressConfig::Never,
            r#use: false,
        }
        .apply(&config)
        .expect("Can't install");

        assert!(config.default_version_dir().exists());
    }

    #[test]
    #[ignore = "hits the real Adoptium API; run manually or against a proxy via the e2e suite"]
    fn test_install_latest() {
        let base_dir = tempfile::tempdir().unwrap();
        let config = FjmConfig::default().with_base_dir(Some(base_dir.path().to_path_buf()));

        Install {
            version: None,
            tool: ToolKind::Java,
            lts: false,
            latest: true,
            progress: ProgressConfig::Never,
            r#use: false,
        }
        .apply(&config)
        .expect("Can't install");

        assert!(config.installations_dir().exists());
    }

    #[test]
    fn test_resolve_major_only_major_falls_back_to_error_when_missing() {
        let available = vec![remote_node_index::IndexedJdkVersion {
            major: 17,
            lts: true,
        }];
        let current = UserVersion::OnlyMajor(99);
        assert!(matches!(
            resolve_major(&current, &available),
            Err(Error::CantFindNodeVersion { .. })
        ));
    }

    #[test]
    fn test_resolve_major_only_major_resolves_when_present() {
        let available = vec![remote_node_index::IndexedJdkVersion {
            major: 17,
            lts: true,
        }];
        let current = UserVersion::OnlyMajor(17);
        assert_eq!(resolve_major(&current, &available).unwrap(), 17);
    }

    #[test]
    fn test_resolve_major_latest_picks_highest_major() {
        let available = vec![
            remote_node_index::IndexedJdkVersion {
                major: 17,
                lts: true,
            },
            remote_node_index::IndexedJdkVersion {
                major: 23,
                lts: false,
            },
        ];
        let current = UserVersion::Full(Version::Latest);
        assert_eq!(resolve_major(&current, &available).unwrap(), 23);
    }

    #[test]
    fn test_asset_resolution_strategy_latest_flag_uses_latest_endpoint() {
        let current = UserVersion::Full(Version::Latest);
        assert_eq!(
            asset_resolution_strategy(&current),
            AssetResolutionStrategy::Latest
        );
    }

    #[test]
    fn test_asset_resolution_strategy_lts_uses_latest_endpoint() {
        let current = UserVersion::Full(Version::Lts(LtsType::Latest));
        assert_eq!(
            asset_resolution_strategy(&current),
            AssetResolutionStrategy::Latest
        );
    }

    #[test]
    fn test_asset_resolution_strategy_exact_semver_honors_exact_patch() {
        let current = UserVersion::from_str("17.0.1").unwrap();
        assert_eq!(
            asset_resolution_strategy(&current),
            AssetResolutionStrategy::ExactPatch(17, 0, 1)
        );
    }

    #[test]
    fn test_asset_resolution_strategy_only_major_has_no_exact_patch() {
        let current = UserVersion::OnlyMajor(17);
        assert_eq!(
            asset_resolution_strategy(&current),
            AssetResolutionStrategy::AnyInMajor
        );
    }

    #[test]
    fn test_resolve_major_lts_delegates_to_lts_type() {
        let available = vec![
            remote_node_index::IndexedJdkVersion {
                major: 17,
                lts: true,
            },
            remote_node_index::IndexedJdkVersion {
                major: 20,
                lts: false,
            },
        ];
        let current = UserVersion::Full(Version::Lts(LtsType::Latest));
        assert_eq!(resolve_major(&current, &available).unwrap(), 17);
    }

    #[test]
    fn test_maven_install_rejects_lts_selector() {
        let base_dir = tempfile::tempdir().unwrap();
        let config = FjmConfig::default().with_base_dir(Some(base_dir.path().to_path_buf()));
        let err = Install {
            version: Some(UserVersion::Full(Version::Lts(LtsType::Latest))),
            tool: ToolKind::Maven,
            lts: false,
            latest: false,
            progress: ProgressConfig::Never,
            r#use: false,
        }
        .apply(&config)
        .unwrap_err();
        assert!(matches!(err, Error::LtsNotSupportedForMaven));
    }
}
