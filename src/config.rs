use crate::arch::Arch;
use crate::directories::Directories;
use crate::log_level::LogLevel;
use crate::path_ext::PathExt;
use crate::tool_kind::ToolKind;
use crate::version_file_strategy::VersionFileStrategy;
use url::Url;

#[derive(clap::Parser, Debug, Clone)]
pub struct FjmConfig {
    /// Adoptium (Eclipse Temurin) API base URL override
    #[clap(
        long,
        env = "FJM_JDK_DIST_MIRROR",
        default_value = "https://api.adoptium.net",
        global = true,
        hide_env_values = true
    )]
    pub jdk_dist_mirror: Url,

    /// Maven Central (or a mirror) base URL override, used to resolve and
    /// download Maven distributions. Symmetric to `--jdk-dist-mirror`.
    #[clap(
        long,
        env = "FJM_MAVEN_DIST_MIRROR",
        default_value = "https://repo.maven.apache.org/maven2",
        global = true,
        hide_env_values = true
    )]
    pub maven_dist_mirror: Url,

    /// The root directory of fjm installations.
    #[clap(
        long = "fjm-dir",
        env = "FJM_DIR",
        global = true,
        hide_env_values = true
    )]
    pub base_dir: Option<std::path::PathBuf>,

    /// Where the current tools' activation slots are stored.
    /// This value will be populated automatically by evaluating
    /// `fjm env` in your shell profile. Read more about it using `fjm help env`
    #[clap(long, env = "FJM_MULTISHELL_PATH", hide_env_values = true, hide = true)]
    multishell_path: Option<std::path::PathBuf>,

    /// The log level of fjm commands
    #[clap(
        long,
        env = "FJM_LOGLEVEL",
        default_value_t,
        global = true,
        hide_env_values = true
    )]
    log_level: LogLevel,

    /// Override the architecture of the installed JDK binary.
    /// Defaults to arch of fjm binary.
    #[clap(
        long,
        env = "FJM_ARCH",
        default_value_t,
        global = true,
        hide_env_values = true,
        hide_default_value = true
    )]
    pub arch: Arch,

    /// A strategy for how to resolve the JDK version. Used whenever `fjm use` or `fjm install` is
    /// called without a version, or when `--use-on-cd` is configured on evaluation.
    #[clap(
        long,
        env = "FJM_VERSION_FILE_STRATEGY",
        default_value_t,
        global = true,
        hide_env_values = true
    )]
    version_file_strategy: VersionFileStrategy,

    #[clap(skip)]
    directories: Directories,
}

impl Default for FjmConfig {
    fn default() -> Self {
        Self {
            jdk_dist_mirror: Url::parse("https://api.adoptium.net").unwrap(),
            maven_dist_mirror: Url::parse("https://repo.maven.apache.org/maven2").unwrap(),
            base_dir: None,
            multishell_path: None,
            log_level: LogLevel::Info,
            arch: Arch::default(),
            version_file_strategy: VersionFileStrategy::default(),
            directories: Directories::default(),
        }
    }
}

impl FjmConfig {
    pub fn version_file_strategy(&self) -> VersionFileStrategy {
        self.version_file_strategy
    }

    pub fn multishell_path(&self) -> Option<&std::path::Path> {
        match &self.multishell_path {
            None => None,
            Some(v) => Some(v.as_path()),
        }
    }

    /// The per-tool activation slot beneath the multishell instance
    /// directory, e.g. `<multishell_path>/java`, `<multishell_path>/maven`.
    /// Each slot is an independent symlink to that tool's active
    /// installation, giving every `ToolKind` its own `env_var_name()` export
    /// and PATH entry without disturbing the others.
    pub fn multishell_path_for(&self, tool: ToolKind) -> Option<std::path::PathBuf> {
        self.multishell_path().map(|p| p.join(tool.as_str()))
    }

    pub fn log_level(&self) -> LogLevel {
        self.log_level
    }

    pub fn base_dir_with_default(&self) -> std::path::PathBuf {
        if let Some(dir) = &self.base_dir {
            return dir.clone();
        }

        self.directories.default_base_dir()
    }

    /// The distribution mirror/index base URL for `tool`.
    pub fn dist_mirror_for(&self, tool: ToolKind) -> &Url {
        match tool {
            ToolKind::Java => &self.jdk_dist_mirror,
            ToolKind::Maven => &self.maven_dist_mirror,
        }
    }

    /// The Java installations directory. Kept as a thin `ToolKind::Java`
    /// convenience wrapper around [`Self::installations_dir_for`], since
    /// most of the codebase predates the multi-tool dimension.
    pub fn installations_dir(&self) -> std::path::PathBuf {
        self.installations_dir_for(ToolKind::Java)
    }

    pub fn installations_dir_for(&self, tool: ToolKind) -> std::path::PathBuf {
        self.base_dir_with_default()
            .join(tool.dir_name())
            .ensure_exists_silently()
    }

    pub fn default_version_dir(&self) -> std::path::PathBuf {
        self.default_version_dir_for(ToolKind::Java)
    }

    pub fn default_version_dir_for(&self, tool: ToolKind) -> std::path::PathBuf {
        self.aliases_dir_for(tool).join("default")
    }

    pub fn aliases_dir(&self) -> std::path::PathBuf {
        self.aliases_dir_for(ToolKind::Java)
    }

    pub fn aliases_dir_for(&self, tool: ToolKind) -> std::path::PathBuf {
        let dir_name = match tool {
            ToolKind::Java => "aliases".to_string(),
            ToolKind::Maven => format!("aliases-{tool}"),
        };
        self.base_dir_with_default()
            .join(dir_name)
            .ensure_exists_silently()
    }

    pub fn multishell_storage(&self) -> std::path::PathBuf {
        self.directories.multishell_storage()
    }

    #[cfg(test)]
    pub fn with_base_dir(mut self, base_dir: Option<std::path::PathBuf>) -> Self {
        self.base_dir = base_dir;
        self
    }

    pub fn with_multishell_path(mut self, multishell_path: std::path::PathBuf) -> Self {
        self.multishell_path = Some(multishell_path);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_keeps_singular_aliases_dir_name() {
        let base_dir = tempfile::tempdir().unwrap();
        let config = FjmConfig::default().with_base_dir(Some(base_dir.path().to_path_buf()));
        assert_eq!(
            config.aliases_dir_for(ToolKind::Java),
            base_dir.path().join("aliases")
        );
    }

    #[test]
    fn test_maven_gets_its_own_aliases_dir() {
        let base_dir = tempfile::tempdir().unwrap();
        let config = FjmConfig::default().with_base_dir(Some(base_dir.path().to_path_buf()));
        assert_eq!(
            config.aliases_dir_for(ToolKind::Maven),
            base_dir.path().join("aliases-maven")
        );
    }

    #[test]
    fn test_multishell_path_for_is_scoped_per_tool() {
        let config = FjmConfig::default()
            .with_multishell_path(std::path::PathBuf::from("/tmp/fjm-multishell"));
        assert_eq!(
            config.multishell_path_for(ToolKind::Java),
            Some(std::path::PathBuf::from("/tmp/fjm-multishell/java"))
        );
        assert_eq!(
            config.multishell_path_for(ToolKind::Maven),
            Some(std::path::PathBuf::from("/tmp/fjm-multishell/maven"))
        );
    }

    #[test]
    fn test_dist_mirror_for_dispatches_by_tool() {
        let config = FjmConfig::default();
        assert_eq!(
            config.dist_mirror_for(ToolKind::Java).as_str(),
            "https://api.adoptium.net/"
        );
        assert_eq!(
            config.dist_mirror_for(ToolKind::Maven).as_str(),
            "https://repo.maven.apache.org/maven2"
        );
    }
}
