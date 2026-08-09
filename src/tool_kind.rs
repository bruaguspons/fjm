//! The compile-time-fixed set of tools fjm can manage.
//!
//! `ToolKind` is threaded through config, version parsing, version-file
//! discovery, alias/install listing and activation as a plain enum dispatched
//! via `match` (not a runtime map or `dyn` trait object) — the tool set is
//! small and known at compile time, so adding a third tool (e.g. Gradle) is a
//! one-arm-per-match, one-new-module change, not a rearchitecture.

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
pub enum ToolKind {
    Java,
    Maven,
}

impl Default for ToolKind {
    /// `java` is the implicit default tool for bare (`--tool`-less) commands.
    /// This is TRANSITIONAL for migration convenience, not a permanent
    /// guarantee — see the spec's "Default Tool Behavior for Bare Commands"
    /// requirement.
    fn default() -> Self {
        Self::Java
    }
}

impl ToolKind {
    /// All tool kinds fjm supports, in a stable order. Drives the
    /// multi-slot activation loop in `commands::env` and the all-tools
    /// default output of `current`/`list`.
    pub fn all() -> &'static [ToolKind] {
        &[ToolKind::Java, ToolKind::Maven]
    }

    /// The env var fjm exports for this tool's active installation root.
    pub fn env_var_name(self) -> &'static str {
        match self {
            Self::Java => "JAVA_HOME",
            Self::Maven => "MAVEN_HOME",
        }
    }

    /// The per-directory version-pin filename fjm looks for.
    pub fn version_file_name(self) -> &'static str {
        match self {
            Self::Java => ".java-version",
            Self::Maven => ".maven-version",
        }
    }

    /// The on-disk directory name (under fjm's base dir) installations for
    /// this tool are stored under.
    ///
    /// `Java` deliberately keeps the legacy `node-versions` name inherited
    /// from the fnm scaffold this project was bootstrapped from — renaming it
    /// would force a migration story that's explicitly out of scope for this
    /// change.
    pub fn dir_name(self) -> &'static str {
        match self {
            Self::Java => "node-versions",
            Self::Maven => "maven-versions",
        }
    }

    /// The short, lowercase identifier used in `--tool`, multishell slot
    /// paths, and multi-tool `current`/`list` output.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Java => "java",
            Self::Maven => "maven",
        }
    }
}

impl std::fmt::Display for ToolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_tool_is_java() {
        assert_eq!(ToolKind::default(), ToolKind::Java);
    }

    #[test]
    fn test_all_contains_both_tools() {
        assert_eq!(ToolKind::all(), &[ToolKind::Java, ToolKind::Maven]);
    }

    #[test]
    fn test_env_var_names() {
        assert_eq!(ToolKind::Java.env_var_name(), "JAVA_HOME");
        assert_eq!(ToolKind::Maven.env_var_name(), "MAVEN_HOME");
    }

    #[test]
    fn test_version_file_names() {
        assert_eq!(ToolKind::Java.version_file_name(), ".java-version");
        assert_eq!(ToolKind::Maven.version_file_name(), ".maven-version");
    }

    #[test]
    fn test_java_keeps_legacy_dir_name() {
        assert_eq!(ToolKind::Java.dir_name(), "node-versions");
    }

    #[test]
    fn test_maven_gets_its_own_dir_name() {
        assert_eq!(ToolKind::Maven.dir_name(), "maven-versions");
    }
}
