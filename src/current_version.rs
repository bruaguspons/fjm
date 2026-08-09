use thiserror::Error;

use crate::config::FjmConfig;
use crate::system_version;
use crate::tool_kind::ToolKind;
use crate::version::Version;

#[allow(
    dead_code,
    reason = "ToolKind::Java-default convenience wrapper around current_version_for; kept for symmetry with the rest of the tool-scoped API"
)]
pub fn current_version(config: &FjmConfig) -> Result<Option<Version>, Error> {
    current_version_for(config, ToolKind::Java)
}

pub fn current_version_for(config: &FjmConfig, tool: ToolKind) -> Result<Option<Version>, Error> {
    let multishell_path = config
        .multishell_path_for(tool)
        .ok_or(Error::EnvNotApplied)?;

    if multishell_path.read_link().ok() == Some(system_version::path()) {
        return Ok(Some(Version::Bypassed));
    }

    if let Ok(resolved_path) = std::fs::canonicalize(&multishell_path) {
        let installation_path = resolved_path
            .parent()
            .expect("multishell path can't be in the root");
        let file_name = installation_path
            .file_name()
            .expect("Can't get filename")
            .to_str()
            .expect("Invalid OS string");
        let version = Version::parse(file_name, tool).map_err(|source| Error::VersionError {
            source,
            version: file_name.to_string(),
        })?;
        Ok(Some(version))
    } else {
        Ok(None)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("`fjm env` was not applied in this context.\nCan't find fjm's environment variables")]
    EnvNotApplied,
    #[error("Can't read the version as a valid semver")]
    VersionError {
        source: crate::version::Error,
        version: String,
    },
}
