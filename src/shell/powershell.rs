use crate::version_file_strategy::VersionFileStrategy;

use super::Shell;
use indoc::formatdoc;
use std::path::Path;

#[derive(Debug)]
pub struct PowerShell;

impl Shell for PowerShell {
    fn path(&self, path: &Path) -> anyhow::Result<String> {
        let path = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Can't convert path to string"))?;
        // Prepend onto the live `$env:PATH` at script-evaluation time (rather
        // than reading and rewriting this process's own inherited PATH), so
        // that printing one `path()` line per active tool slot correctly
        // accumulates instead of each line clobbering the previous one.
        let separator = if cfg!(windows) { ';' } else { ':' };
        Ok(format!(r#"$env:PATH = "{path}{separator}$env:PATH""#))
    }

    fn set_env_var(&self, name: &str, value: &str) -> String {
        format!(r#"$env:{name} = "{value}""#)
    }

    fn use_on_cd(&self, config: &crate::config::FjmConfig) -> anyhow::Result<String> {
        let version_file_exists_condition = "(Test-Path .java-version)";
        let autoload_hook = match config.version_file_strategy() {
            VersionFileStrategy::Local => formatdoc!(
                r"
                    If ({version_file_exists_condition}) {{ & fjm use --silent-if-unchanged }}
                ",
                version_file_exists_condition = version_file_exists_condition,
            ),
            VersionFileStrategy::Recursive => String::from(r"fjm use --silent-if-unchanged"),
        };
        Ok(formatdoc!(
            r"
                function global:Set-FjmOnLoad {{ {autoload_hook} }}
                function global:Set-LocationWithFjm {{ param($path); if ($path -eq $null) {{Set-Location}} else {{Set-Location $path}}; Set-FjmOnLoad }}
                Set-Alias -Scope global cd_with_fjm Set-LocationWithFjm
                Set-Alias -Option AllScope -Scope global cd Set-LocationWithFjm
            ",
            autoload_hook = autoload_hook
        ))
    }
    fn to_clap_shell(&self) -> clap_complete::Shell {
        clap_complete::Shell::PowerShell
    }
}
