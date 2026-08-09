use super::command::Command;
use super::r#use::Use;
use crate::config::FjmConfig;
use crate::fs::symlink_dir;
use crate::outln;
use crate::path_ext::PathExt;
use crate::shell::{infer_shell, Shell, Shells};
use crate::tool_kind::ToolKind;
use clap::ValueEnum;
use colored::Colorize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::IsTerminal;
use thiserror::Error;

#[derive(clap::Parser, Debug, Default)]
pub struct Env {
    /// The shell syntax to use. Infers when missing.
    #[clap(long)]
    shell: Option<Shells>,
    /// Print JSON instead of shell commands.
    #[clap(long, conflicts_with = "shell")]
    json: bool,
    /// Deprecated. This is the default now.
    #[clap(long, hide = true)]
    multi: bool,
    /// Print the script to change JDK versions every directory change
    #[clap(long)]
    use_on_cd: bool,
}

fn generate_symlink_path() -> String {
    format!(
        "{}_{}_{:010}",
        std::process::id(),
        chrono::Utc::now().timestamp_millis(),
        fastrand::u64(..10_000_000_000),
    )
}

/// Creates the root multishell instance directory for this shell session.
/// Each `ToolKind`'s activation slot lives inside it (see
/// [`activate_default_slots`]).
fn make_multishell_root(config: &FjmConfig) -> Result<std::path::PathBuf, Error> {
    let base_dir = config.multishell_storage().ensure_exists_silently();
    let mut temp_dir = base_dir.join(generate_symlink_path());

    while temp_dir.exists() {
        temp_dir = base_dir.join(generate_symlink_path());
    }

    std::fs::create_dir_all(&temp_dir).map_err(|source| Error::CantCreateSymlink {
        source,
        temp_dir: temp_dir.clone(),
    })?;
    Ok(temp_dir)
}

/// For every `ToolKind`, activates that tool's slot by symlinking
/// `multishell_path_for(tool)` to its `default` alias — even when that
/// alias doesn't exist on disk yet (a dangling symlink), matching this
/// crate's pre-existing single-slot behavior. This is deliberate: `fjm use`
/// only ever repoints this stable slot symlink, it never needs `fjm env` to
/// be re-evaluated, so the slot (and thus PATH) must exist from the start
/// even before anything is installed.
fn activate_default_slots(config: &FjmConfig) -> Result<(), Error> {
    for &tool in ToolKind::all() {
        let default_dir = config.default_version_dir_for(tool);
        let Some(slot_path) = config.multishell_path_for(tool) else {
            continue;
        };
        symlink_dir(default_dir, &slot_path).map_err(|source| Error::CantCreateSymlink {
            source,
            temp_dir: slot_path,
        })?;
    }
    Ok(())
}

/// The bin directories of every tool slot, unconditionally — dangling slots
/// (nothing installed/aliased yet for that tool) are included too, so PATH
/// is stable across `fjm use` repointing the underlying symlink later.
fn active_bin_paths(config: &FjmConfig) -> Vec<std::path::PathBuf> {
    ToolKind::all()
        .iter()
        .filter_map(|&tool| config.multishell_path_for(tool))
        .map(|slot| slot.join("bin"))
        .collect()
}

fn set_path_for_multishell(bin_paths: &[std::path::PathBuf]) {
    let current_path = std::env::var_os("PATH").unwrap_or_default();
    let mut split_paths: Vec<_> = std::env::split_paths(&current_path).collect();
    for bin_path in bin_paths.iter().rev() {
        split_paths.insert(0, bin_path.clone());
    }
    if let Ok(new_path) = std::env::join_paths(split_paths) {
        unsafe {
            std::env::set_var("PATH", new_path);
        }
    }
}

impl Command for Env {
    type Error = Error;

    fn apply(self, config: &FjmConfig) -> Result<(), Self::Error> {
        if self.multi {
            outln!(
                config,
                Error,
                "{} {} is deprecated. This is now the default.",
                "warning:".yellow().bold(),
                "--multi".italic()
            );
        }

        let multishell_path = make_multishell_root(config)?;
        let config_with_multishell = config.clone().with_multishell_path(multishell_path.clone());
        activate_default_slots(&config_with_multishell)?;

        let base_dir = config.base_dir_with_default();

        let mut env_vars = vec![
            (
                "FJM_MULTISHELL_PATH".to_string(),
                multishell_path.to_str().unwrap().to_string(),
            ),
            (
                "FJM_VERSION_FILE_STRATEGY".to_string(),
                config.version_file_strategy().as_str().to_string(),
            ),
            (
                "FJM_DIR".to_string(),
                base_dir.to_str().unwrap().to_string(),
            ),
            (
                "FJM_LOGLEVEL".to_string(),
                config.log_level().as_str().to_string(),
            ),
            (
                "FJM_JDK_DIST_MIRROR".to_string(),
                config.jdk_dist_mirror.as_str().to_string(),
            ),
            (
                "FJM_MAVEN_DIST_MIRROR".to_string(),
                config.maven_dist_mirror.as_str().to_string(),
            ),
            ("FJM_ARCH".to_string(), config.arch.as_str().to_string()),
        ];

        // One env var per currently-active tool slot (e.g. `JAVA_HOME`,
        // `MAVEN_HOME`), additive to the fixed `FJM_*` vars above.
        for &tool in ToolKind::all() {
            if let Some(slot_path) = config_with_multishell.multishell_path_for(tool) {
                if slot_path.exists() {
                    env_vars.push((
                        tool.env_var_name().to_string(),
                        slot_path.to_str().unwrap().to_string(),
                    ));
                }
            }
        }

        if self.json {
            let map: HashMap<&str, &str> = env_vars
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            println!("{}", serde_json::to_string(&map).unwrap());
            return Ok(());
        }

        let shell: Box<dyn Shell> = self
            .shell
            .map(Into::into)
            .or_else(infer_shell)
            .ok_or(Error::CantInferShell)?;

        for bin_path in active_bin_paths(&config_with_multishell) {
            println!("{}", shell.path(&bin_path)?);
        }

        for (name, value) in &env_vars {
            println!("{}", shell.set_env_var(name, value));
        }

        if self.use_on_cd {
            // Call `use` internally for the initial directory, so the shell doesn't
            // need to spawn a subprocess after evaluating the env output.
            set_path_for_multishell(&active_bin_paths(&config_with_multishell));
            let use_cmd = Use {
                version: None,
                tool: ToolKind::Java,
                install_if_missing: false,
                silent_if_unchanged: true,
                info_to_stderr: true,
            };
            let should_force_stderr_color = !std::io::stdout().is_terminal()
                && std::io::stderr().is_terminal()
                && std::env::var_os("NO_COLOR").is_none();
            if should_force_stderr_color {
                colored::control::set_override(true);
            }
            // Ignore errors - if there's no version file, that's fine
            let _ = use_cmd.apply(&config_with_multishell);
            if should_force_stderr_color {
                colored::control::unset_override();
            }

            println!("{}", shell.use_on_cd(config)?);
        }
        if let Some(v) = shell.rehash() {
            println!("{v}");
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(
        "{}\n{}\n{}\n{}",
        "Can't infer shell!",
        "fjm can't infer your shell based on the process tree.",
        "Maybe it is unsupported? we support the following shells:",
        shells_as_string()
    )]
    CantInferShell,
    #[error("Can't create the symlink for multishells at {temp_dir:?}. Maybe there are some issues with permissions for the directory? {source}")]
    CantCreateSymlink {
        #[source]
        source: std::io::Error,
        temp_dir: std::path::PathBuf,
    },
    #[error(transparent)]
    ShellError {
        #[from]
        source: anyhow::Error,
    },
}

fn shells_as_string() -> String {
    Shells::value_variants()
        .iter()
        .map(|x| format!("* {x}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smoke() {
        let config = FjmConfig::default();
        Env {
            #[cfg(windows)]
            shell: Some(Shells::Cmd),
            #[cfg(not(windows))]
            shell: Some(Shells::Bash),
            ..Default::default()
        }
        .call(config);
    }
}
