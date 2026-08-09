use super::command::Command;
use crate::config::FjmConfig;
use crate::current_version::{current_version_for, Error};
use crate::tool_kind::ToolKind;

#[derive(clap::Parser, Debug)]
pub struct Current {
    /// Restrict output to a single tool. When omitted, every tool's active
    /// version is shown.
    #[clap(long, value_enum)]
    tool: Option<ToolKind>,
}

impl Command for Current {
    type Error = Error;

    fn apply(self, config: &FjmConfig) -> Result<(), Self::Error> {
        match self.tool {
            Some(tool) => {
                println!("{}", version_string(config, tool)?);
            }
            None => {
                for tool in ToolKind::all() {
                    println!("{}: {}", tool, version_string(config, *tool)?);
                }
            }
        }
        Ok(())
    }
}

fn version_string(config: &FjmConfig, tool: ToolKind) -> Result<String, Error> {
    Ok(match current_version_for(config, tool)? {
        Some(ver) => ver.v_str(),
        None => "none".into(),
    })
}
