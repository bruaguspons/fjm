use crate::config::FjmConfig;
use crate::remote_maven_index;
use crate::remote_node_index;
use crate::tool_kind::ToolKind;
use crate::user_version::UserVersion;

use colored::Colorize;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct LsRemote {
    /// Which tool's remote index to list.
    #[clap(long, value_enum, default_value_t)]
    tool: ToolKind,

    /// Filter versions by a user-defined version or a semver range
    #[arg(long)]
    filter: Option<UserVersion>,

    /// Show only LTS versions (Java only)
    #[arg(long)]
    lts: bool,

    /// Version sorting order
    #[arg(long, default_value = "asc")]
    sort: SortingMethod,

    /// Only show the latest matching version
    #[arg(long)]
    latest: bool,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
pub enum SortingMethod {
    #[clap(name = "desc")]
    /// Sort versions in descending order (latest to earliest)
    Descending,
    #[clap(name = "asc")]
    /// Sort versions in ascending order (earliest to latest)
    Ascending,
}

/// Drain all elements but the last one
fn truncate_except_latest<T>(list: &mut Vec<T>) {
    let len = list.len();
    if len > 1 {
        list.swap(0, len - 1);
        list.truncate(1);
    }
}

impl super::command::Command for LsRemote {
    type Error = Error;

    fn apply(self, config: &FjmConfig) -> Result<(), Self::Error> {
        match self.tool {
            ToolKind::Java => self.list_java(config),
            ToolKind::Maven => self.list_maven(config),
        }
    }
}

impl LsRemote {
    fn list_java(self, config: &FjmConfig) -> Result<(), Error> {
        let mut all_versions = remote_node_index::list(config.dist_mirror_for(ToolKind::Java))?;

        if self.lts {
            all_versions.retain(|v| v.lts);
        }

        if let Some(filter) = &self.filter {
            all_versions.retain(|v| filter.matches_major(v.major));
        }

        all_versions.sort_by_key(|v| v.major);

        if self.latest {
            truncate_except_latest(&mut all_versions);
        }

        if let SortingMethod::Descending = self.sort {
            all_versions.reverse();
        }

        if all_versions.is_empty() {
            eprintln!("{}", "No versions were found!".red());
            return Ok(());
        }

        for version in &all_versions {
            print!("{}", version.major);
            if version.lts {
                print!("{}", " (LTS)".cyan());
            }
            println!();
        }

        Ok(())
    }

    fn list_maven(self, config: &FjmConfig) -> Result<(), Error> {
        let mut all_versions = remote_maven_index::list(config.dist_mirror_for(ToolKind::Maven))?;

        all_versions.sort();

        if self.latest {
            truncate_except_latest(&mut all_versions);
        }

        if let SortingMethod::Descending = self.sort {
            all_versions.reverse();
        }

        if all_versions.is_empty() {
            eprintln!("{}", "No versions were found!".red());
            return Ok(());
        }

        for version in &all_versions {
            println!("{}", version.v_str());
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    RemoteListing {
        #[from]
        source: remote_node_index::Error,
    },
    #[error(transparent)]
    RemoteMavenListing {
        #[from]
        source: remote_maven_index::Error,
    },
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_truncate_except_latest() {
        let mut list = vec![1, 2, 3, 4, 5];
        truncate_except_latest(&mut list);
        assert_eq!(list, vec![5]);

        let mut list: Vec<()> = vec![];
        truncate_except_latest(&mut list);
        assert_eq!(list, vec![]);

        let mut list = vec![1];
        truncate_except_latest(&mut list);
        assert_eq!(list, vec![1]);
    }
}
