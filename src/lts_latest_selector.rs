use crate::lts::LtsType;
use crate::user_version::UserVersion;
use crate::version::Version;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
#[error(
    "Too many versions provided. Please don't use --lts/--latest together with a version string."
)]
pub struct ConflictingVersionSelectors;

/// Resolves an explicit version argument together with the `--lts`/`--latest` sugar flags
/// (mirroring `fjm install`'s `--lts`/`--latest`) into a single optional [`UserVersion`].
///
/// `version`, `lts`, and `latest` are expected to already be mutually exclusive at the clap
/// level (`conflicts_with_all`); this only rejects the case where more than one of them somehow
/// arrives set anyway.
pub fn resolve(
    version: Option<UserVersion>,
    lts: bool,
    latest: bool,
) -> Result<Option<UserVersion>, ConflictingVersionSelectors> {
    match (version, lts, latest) {
        (v, false, false) => Ok(v),
        (None, true, false) => Ok(Some(UserVersion::Full(Version::Lts(LtsType::Latest)))),
        (None, false, true) => Ok(Some(UserVersion::Full(Version::Latest))),
        _ => Err(ConflictingVersionSelectors),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_flags_passes_version_through() {
        let version = Some(UserVersion::Full(Version::Latest));
        assert_eq!(resolve(version.clone(), false, false), Ok(version));
        assert_eq!(resolve(None, false, false), Ok(None));
    }

    #[test]
    fn test_lts_flag_resolves_to_lts_latest() {
        assert_eq!(
            resolve(None, true, false),
            Ok(Some(UserVersion::Full(Version::Lts(LtsType::Latest))))
        );
    }

    #[test]
    fn test_latest_flag_resolves_to_latest() {
        assert_eq!(
            resolve(None, false, true),
            Ok(Some(UserVersion::Full(Version::Latest)))
        );
    }

    #[test]
    fn test_version_with_lts_flag_conflicts() {
        let version = Some(UserVersion::Full(Version::Latest));
        assert_eq!(
            resolve(version, true, false),
            Err(ConflictingVersionSelectors)
        );
    }

    #[test]
    fn test_lts_and_latest_together_conflicts() {
        assert_eq!(resolve(None, true, true), Err(ConflictingVersionSelectors));
    }
}
