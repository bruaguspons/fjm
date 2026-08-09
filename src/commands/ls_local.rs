use crate::alias::{list_aliases, StoredAlias};
use crate::config::FjmConfig;
use crate::current_version::current_version_for;
use crate::tool_kind::ToolKind;
use crate::version::Version;
use colored::Colorize;
use std::collections::HashMap;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct LsLocal {
    /// Restrict listing to a single tool. When omitted, every tool's
    /// installed versions are listed.
    #[clap(long, value_enum)]
    tool: Option<ToolKind>,
}

impl super::command::Command for LsLocal {
    type Error = Error;

    fn apply(self, config: &FjmConfig) -> Result<(), Self::Error> {
        let tools: Vec<ToolKind> = match self.tool {
            Some(tool) => vec![tool],
            None => ToolKind::all().to_vec(),
        };

        let multiple_tools = tools.len() > 1;

        for tool in tools {
            if multiple_tools {
                println!("{}:", tool.to_string().bold());
            }
            list_for_tool(config, tool)?;
        }

        Ok(())
    }
}

fn list_for_tool(config: &FjmConfig, tool: ToolKind) -> Result<(), Error> {
    let base_dir = config.installations_dir_for(tool);
    let mut versions = crate::installed_versions::list_for_tool(base_dir, tool)
        .map_err(|source| Error::CantListLocallyInstalledVersion { source })?;
    versions.insert(0, Version::Bypassed);
    versions.sort();
    let aliases_hash =
        generate_aliases_hash(config, tool).map_err(|source| Error::CantReadAliases { source })?;
    let curr_version = current_version_for(config, tool).ok().flatten();

    for version in versions {
        let version_aliases = match aliases_hash.get(&version.v_str()) {
            None => String::new(),
            Some(versions) => {
                let version_string = versions
                    .iter()
                    .map(StoredAlias::name)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(" {}", version_string.dimmed())
            }
        };

        let version_str = format!("* {version}{version_aliases}");

        if curr_version == Some(version) {
            println!("{}", version_str.cyan());
        } else {
            println!("{version_str}");
        }
    }
    Ok(())
}

fn generate_aliases_hash(
    config: &FjmConfig,
    tool: ToolKind,
) -> std::io::Result<HashMap<String, Vec<StoredAlias>>> {
    let mut aliases = list_aliases(config, tool)?;
    let mut hashmap: HashMap<String, Vec<StoredAlias>> = HashMap::with_capacity(aliases.len());
    for alias in aliases.drain(..) {
        if let Some(value) = hashmap.get_mut(alias.s_ver()) {
            value.push(alias);
        } else {
            hashmap.insert(alias.s_ver().into(), vec![alias]);
        }
    }
    Ok(hashmap)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Can't list locally installed versions: {}", source)]
    CantListLocallyInstalledVersion {
        source: crate::installed_versions::Error,
    },
    #[error("Can't read aliases: {}", source)]
    CantReadAliases { source: std::io::Error },
}
