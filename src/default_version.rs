use crate::config::FjmConfig;
use crate::tool_kind::ToolKind;
use crate::version::Version;
use std::str::FromStr;

#[allow(
    dead_code,
    reason = "ToolKind::Java-default convenience wrapper around find_default_version_for; kept for symmetry with the rest of the tool-scoped API"
)]
pub fn find_default_version(config: &FjmConfig) -> Option<Version> {
    find_default_version_for(config, ToolKind::Java)
}

pub fn find_default_version_for(config: &FjmConfig, tool: ToolKind) -> Option<Version> {
    if let Ok(version_path) = config.default_version_dir_for(tool).canonicalize() {
        let file_name = version_path.parent()?.file_name()?;
        Version::from_str(file_name.to_str()?).ok()?.into()
    } else {
        Some(Version::Alias("default".into()))
    }
}
