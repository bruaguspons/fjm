use crate::arch::Arch;
use crate::directories::Directories;
use crate::log_level::LogLevel;
use crate::path_ext::PathExt;
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

    /// The root directory of fjm installations.
    #[clap(
        long = "fjm-dir",
        env = "FJM_DIR",
        global = true,
        hide_env_values = true
    )]
    pub base_dir: Option<std::path::PathBuf>,

    /// Where the current JDK version link is stored.
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

    pub fn log_level(&self) -> LogLevel {
        self.log_level
    }

    pub fn base_dir_with_default(&self) -> std::path::PathBuf {
        if let Some(dir) = &self.base_dir {
            return dir.clone();
        }

        self.directories.default_base_dir()
    }

    pub fn installations_dir(&self) -> std::path::PathBuf {
        self.base_dir_with_default()
            .join("node-versions")
            .ensure_exists_silently()
    }

    pub fn default_version_dir(&self) -> std::path::PathBuf {
        self.aliases_dir().join("default")
    }

    pub fn aliases_dir(&self) -> std::path::PathBuf {
        self.base_dir_with_default()
            .join("aliases")
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
