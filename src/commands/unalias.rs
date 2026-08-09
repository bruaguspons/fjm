use super::command::Command;
use crate::choose_version_for_user_input::choose_version_for_user_input_for_tool;
use crate::config::FjmConfig;
use crate::fs::remove_symlink_dir;
use crate::tool_kind::ToolKind;
use crate::user_version::UserVersion;
use crate::version::Version;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Unalias {
    pub(crate) requested_alias: String,

    /// Which tool the alias belongs to.
    #[clap(long, value_enum, default_value_t)]
    pub tool: ToolKind,
}

impl Command for Unalias {
    type Error = Error;

    fn apply(self, config: &FjmConfig) -> Result<(), Self::Error> {
        let requested_version = choose_version_for_user_input_for_tool(
            &UserVersion::Full(Version::Alias(self.requested_alias.clone())),
            config,
            self.tool,
        )
        .ok()
        .flatten()
        .ok_or(Error::AliasNotFound {
            requested_alias: self.requested_alias,
        })?;

        remove_symlink_dir(requested_version.path())
            .map_err(|source| Error::CantDeleteSymlink { source })?;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Can't delete symlink: {}", source)]
    CantDeleteSymlink { source: std::io::Error },
    #[error("Requested alias {} not found", requested_alias)]
    AliasNotFound { requested_alias: String },
}
