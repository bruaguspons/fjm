use crate::alias;
use crate::config;
use crate::lts::LtsType;
use crate::system_version;
use crate::tool_kind::ToolKind;
use std::str::FromStr;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub enum Version {
    Semver(node_semver::Version),
    Lts(LtsType),
    Alias(String),
    Latest,
    Bypassed,
}

fn first_letter_is_number(s: &str) -> bool {
    s.chars().next().is_some_and(|x| x.is_ascii_digit())
}

/// Detects the pre-JEP223 legacy JDK version format, e.g. `1.8.0_311`: a
/// dotted numeric version segment followed by an `_`-separated numeric
/// update number.
fn is_legacy_update_format(version_plain: &str) -> bool {
    match version_plain.find('_') {
        Some(idx) => {
            let (before, after) = version_plain.split_at(idx);
            let after = &after[1..];
            !before.is_empty()
                && !after.is_empty()
                && before.chars().all(|c| c.is_ascii_digit() || c == '.')
                && after.chars().all(|c| c.is_ascii_digit())
        }
        None => false,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Semver(#[from] node_semver::SemverError),
    #[error("legacy JDK version format (e.g. `1.8.0_311`) is not supported yet, see PRD §10")]
    LegacyFormatNotSupported,
    #[error("invalid LTS version `{0}`: JDK LTS releases are numeric majors, e.g. `lts-17` or `lts-latest`")]
    InvalidLtsMajor(String),
}

impl Version {
    /// Parses a user- or index-supplied version string for `tool`.
    ///
    /// The JDK-only rules (LTS selectors, rejection of the legacy
    /// `1.8.0_311`-style update format) only apply when `tool` is
    /// [`ToolKind::Java`] — Maven has no LTS concept and no legacy update
    /// format, so those branches are skipped and Maven version strings (e.g.
    /// `3.9.9`) pass straight through to the semver parser.
    pub fn parse<S: AsRef<str>>(version_str: S, tool: ToolKind) -> Result<Self, Error> {
        let lowercased = version_str.as_ref().to_lowercase();
        if lowercased == system_version::display_name() {
            Ok(Self::Bypassed)
        } else if tool == ToolKind::Java
            && (lowercased.starts_with("lts-") || lowercased.starts_with("lts/"))
        {
            let lts_str = &lowercased[4..];
            let lts_type = lts_str
                .parse::<LtsType>()
                .map_err(|_| Error::InvalidLtsMajor(lts_str.to_string()))?;
            Ok(Self::Lts(lts_type))
        } else if first_letter_is_number(lowercased.trim_start_matches('v')) {
            let version_plain = lowercased.trim_start_matches('v');
            if tool == ToolKind::Java && is_legacy_update_format(version_plain) {
                return Err(Error::LegacyFormatNotSupported);
            }
            let sver = node_semver::Version::parse(version_plain)?;
            Ok(Self::Semver(sver))
        } else {
            Ok(Self::Alias(lowercased))
        }
    }

    pub fn alias_name(&self) -> Option<String> {
        match self {
            l @ (Self::Lts(_) | Self::Alias(_)) => Some(l.v_str()),
            _ => None,
        }
    }

    pub fn find_aliases(
        &self,
        config: &config::FjmConfig,
        tool: ToolKind,
    ) -> std::io::Result<Vec<alias::StoredAlias>> {
        let aliases = alias::list_aliases(config, tool)?
            .drain(..)
            .filter(|alias| alias.s_ver() == self.v_str())
            .collect();
        Ok(aliases)
    }

    pub fn v_str(&self) -> String {
        format!("{self}")
    }

    pub fn installation_path(
        &self,
        config: &config::FjmConfig,
        tool: ToolKind,
    ) -> std::path::PathBuf {
        match self {
            Self::Bypassed => system_version::path(),
            v @ (Self::Lts(_) | Self::Alias(_) | Self::Latest) => {
                config.aliases_dir_for(tool).join(v.alias_name().unwrap())
            }
            v @ Self::Semver(_) => config
                .installations_dir_for(tool)
                .join(v.v_str())
                .join("installation"),
        }
    }

    pub fn root_path(
        &self,
        config: &config::FjmConfig,
        tool: ToolKind,
    ) -> Option<std::path::PathBuf> {
        let path = self.installation_path(config, tool);
        let mut canon_path = path.canonicalize().ok()?;
        canon_path.pop();
        Some(canon_path)
    }
}

// TODO: add a trait called BinPath that &Path and PathBuf implements
// which adds the `.bin_path()` which works both on windows and unix :)

impl<'de> serde::Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let version_str = String::deserialize(deserializer)?;
        Version::parse(version_str, ToolKind::Java).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bypassed => write!(f, "{}", system_version::display_name()),
            Self::Lts(lts) => write!(f, "lts-{lts}"),
            Self::Semver(semver) => write!(f, "v{semver}"),
            Self::Alias(alias) => write!(f, "{alias}"),
            Self::Latest => write!(f, "latest"),
        }
    }
}

impl FromStr for Version {
    type Err = Error;
    /// Defaults to [`ToolKind::Java`] — `java` is the transitional implicit
    /// default tool, and `FromStr` has no way to receive a tool context.
    /// Callers that know their tool should call [`Version::parse`] directly.
    fn from_str(s: &str) -> Result<Version, Self::Err> {
        Self::parse(s, ToolKind::Java)
    }
}

impl PartialEq<node_semver::Version> for Version {
    fn eq(&self, other: &node_semver::Version) -> bool {
        match self {
            Self::Bypassed | Self::Lts(_) | Self::Alias(_) | Self::Latest => false,
            Self::Semver(v) => v == other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_format_is_not_supported() {
        assert!(matches!(
            Version::parse("1.8.0_311", ToolKind::Java),
            Err(Error::LegacyFormatNotSupported)
        ));
    }

    #[test]
    fn test_modern_jdk_version_still_parses() {
        assert!(matches!(
            Version::parse("17.0.2", ToolKind::Java),
            Ok(Version::Semver(_))
        ));
    }

    #[test]
    fn test_non_numeric_lts_is_a_hard_parse_error() {
        assert!(matches!(
            Version::parse("lts-jod", ToolKind::Java),
            Err(Error::InvalidLtsMajor(s)) if s == "jod"
        ));
    }

    #[test]
    fn test_numeric_lts_parses_into_major() {
        assert!(matches!(
            Version::parse("lts-17", ToolKind::Java),
            Ok(Version::Lts(LtsType::Major(17)))
        ));
    }

    #[test]
    fn test_bare_lts_latest_parses() {
        assert!(matches!(
            Version::parse("lts-latest", ToolKind::Java),
            Ok(Version::Lts(LtsType::Latest))
        ));
    }

    #[test]
    fn test_maven_rejects_lts_selector_as_a_literal_alias() {
        // Maven has no LTS concept: `lts-17` for Maven is just an (unusual)
        // alias name, not a parse error and not an `LtsType`.
        assert!(matches!(
            Version::parse("lts-17", ToolKind::Maven),
            Ok(Version::Alias(s)) if s == "lts-17"
        ));
    }

    #[test]
    fn test_maven_accepts_legacy_update_shaped_string() {
        // The `1.8.0_311` legacy-update-format rejection is JDK-specific;
        // for Maven, a `_`-suffixed string like this isn't a valid semver
        // anyway, so it surfaces as a normal semver parse error, not
        // `LegacyFormatNotSupported`.
        assert!(!matches!(
            Version::parse("1.8.0_311", ToolKind::Maven),
            Err(Error::LegacyFormatNotSupported)
        ));
    }

    #[test]
    fn test_maven_plain_version_parses() {
        assert!(matches!(
            Version::parse("3.9.9", ToolKind::Maven),
            Ok(Version::Semver(_))
        ));
    }
}
