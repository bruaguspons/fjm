use super::alias::Alias;
use super::command::Command;
use crate::alias::get_alias_by_name;
use crate::config::FjmConfig;
use crate::tool_kind::ToolKind;
use crate::user_version::UserVersion;

#[derive(clap::Parser, Debug)]
pub struct Default {
    version: Option<UserVersion>,

    /// Which tool to set/read the default version for.
    #[clap(long, value_enum, default_value_t)]
    tool: ToolKind,
}

impl Command for Default {
    type Error = super::alias::Error;

    fn apply(self, config: &FjmConfig) -> Result<(), Self::Error> {
        match self.version {
            Some(version) => Alias {
                name: "default".into(),
                to_version: version,
                tool: self.tool,
            }
            .apply(config),
            None => match get_alias_by_name(config, "default", self.tool) {
                Some(alias) => {
                    println!("{}", alias.s_ver());
                    Ok(())
                }
                None => Err(Self::Error::DefaultAliasDoesNotExist),
            },
        }
    }

    fn handle_error(err: Self::Error, config: &FjmConfig) {
        Alias::handle_error(err, config);
    }
}
