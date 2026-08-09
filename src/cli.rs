use crate::commands;
use crate::commands::command::Command;
use crate::config::FjmConfig;
use clap::Parser;

#[derive(clap::Parser, Debug)]
pub enum SubCommand {
    /// List all remote JDK versions
    #[clap(name = "list-remote", bin_name = "list-remote", visible_aliases = &["ls-remote"])]
    LsRemote(commands::ls_remote::LsRemote),

    /// List all locally installed JDK versions
    #[clap(name = "list", bin_name = "list", visible_aliases = &["ls"])]
    LsLocal(commands::ls_local::LsLocal),

    /// Install a new JDK version
    #[clap(name = "install", bin_name = "install", visible_aliases = &["i"])]
    Install(commands::install::Install),

    /// Change JDK version for the current shell session only
    ///
    /// This only affects the shell it's run in — closing the terminal and opening a new one
    /// loses it. To change what version new shells start with, use `fjm default` instead.
    #[clap(name = "use", bin_name = "use", verbatim_doc_comment)]
    Use(commands::r#use::Use),

    /// Print and set up required environment variables for fjm
    ///
    /// This command generates a series of shell commands that
    /// should be evaluated by your shell to create a fjm-ready environment.
    ///
    /// Each shell has its own syntax of evaluating a dynamic expression.
    /// For example, evaluating fjm on Bash and Zsh would look like `eval "$(fjm env --shell bash)"`.
    /// In Fish, evaluating would look like `fjm env --shell fish | source`
    #[clap(name = "env", bin_name = "env")]
    Env(commands::env::Env),

    /// Print shell completions to stdout
    #[clap(name = "completions", bin_name = "completions")]
    Completions(commands::completions::Completions),

    /// Alias a version to a common name
    #[clap(name = "alias", bin_name = "alias")]
    Alias(commands::alias::Alias),

    /// Remove an alias definition
    #[clap(name = "unalias", bin_name = "unalias")]
    Unalias(commands::unalias::Unalias),

    /// Set a version as the default version or get the current default version.
    ///
    /// This is a shorthand for `fjm alias VERSION default`. Unlike `fjm use`, this persists:
    /// it's what every new shell starts with (as long as `fjm env` is set up in your shell's
    /// startup file — see the README's Shell Setup section).
    #[clap(name = "default", bin_name = "default", verbatim_doc_comment)]
    Default(commands::default::Default),

    /// Print the current JDK version
    #[clap(name = "current", bin_name = "current")]
    Current(commands::current::Current),

    /// Run a command within fjm context
    ///
    /// Example:
    /// --------
    /// fjm exec --using=17.0.2 java --version
    /// => 17.0.2
    #[clap(name = "exec", bin_name = "exec", verbatim_doc_comment)]
    Exec(commands::exec::Exec),

    /// Uninstall a JDK version
    ///
    /// > Warning: when providing an alias, it will remove the JDK version the alias
    /// > is pointing to, along with the other aliases that point to the same version.
    #[clap(name = "uninstall", bin_name = "uninstall", visible_aliases = &["uni"])]
    Uninstall(commands::uninstall::Uninstall),
}

impl SubCommand {
    pub fn call(self, config: FjmConfig) {
        match self {
            Self::LsLocal(cmd) => cmd.call(config),
            Self::LsRemote(cmd) => cmd.call(config),
            Self::Install(cmd) => cmd.call(config),
            Self::Env(cmd) => cmd.call(config),
            Self::Use(cmd) => cmd.call(config),
            Self::Completions(cmd) => cmd.call(config),
            Self::Alias(cmd) => cmd.call(config),
            Self::Default(cmd) => cmd.call(config),
            Self::Current(cmd) => cmd.call(config),
            Self::Exec(cmd) => cmd.call(config),
            Self::Uninstall(cmd) => cmd.call(config),
            Self::Unalias(cmd) => cmd.call(config),
        }
    }
}

/// A fast and simple JDK manager.
#[derive(clap::Parser, Debug)]
#[clap(name = "fjm", version = env!("CARGO_PKG_VERSION"), bin_name = "fjm")]
pub struct Cli {
    #[clap(flatten)]
    pub config: FjmConfig,
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

pub fn parse() -> Cli {
    Cli::parse()
}
