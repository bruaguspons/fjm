use super::shell::Shell;
use std::path::Path;

#[derive(Debug)]
pub struct WindowsCmd;

impl Shell for WindowsCmd {
    fn to_clap_shell(&self) -> clap_complete::Shell {
        // TODO: move to Option
        panic!("Shell completion is not supported for Windows Command Prompt. Maybe try using PowerShell for a better experience?");
    }

    fn path(&self, path: &Path) -> anyhow::Result<String> {
        let path = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Can't convert path to string"))?;
        // Prepend onto the live `%PATH%` at script-evaluation time (rather
        // than reading and rewriting this process's own inherited PATH), so
        // that printing one `path()` line per active tool slot correctly
        // accumulates instead of each line clobbering the previous one.
        Ok(format!("SET PATH={path};%PATH%"))
    }

    fn set_env_var(&self, name: &str, value: &str) -> String {
        format!("SET {name}={value}")
    }

    fn use_on_cd(&self, config: &crate::config::FjmConfig) -> anyhow::Result<String> {
        let path = config.base_dir_with_default().join("cd.cmd");
        create_cd_file_at(&path).map_err(|source| {
            anyhow::anyhow!(
                "Can't create cd.cmd file for use-on-cd at {}: {}",
                path.display(),
                source
            )
        })?;
        let path = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Can't read path to cd.cmd"))?;
        Ok(format!("doskey cd=\"{path}\" $*"))
    }
}

fn create_cd_file_at(path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Write;
    let cmd_contents = include_bytes!("./cd.cmd");
    let mut file = std::fs::File::create(path)?;
    file.write_all(cmd_contents)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn use_on_cd_quotes_macro_path() {
        let base_dir = std::env::temp_dir().join("fjm cmd test with spaces");
        std::fs::create_dir_all(&base_dir).unwrap();
        let config = crate::config::FjmConfig::default().with_base_dir(Some(base_dir.clone()));
        let output = WindowsCmd.use_on_cd(&config).unwrap();

        assert!(output.starts_with("doskey cd=\""));
        assert!(output.ends_with("\" $*"));
        assert!(output.contains("with spaces"));
        assert!(output.contains("cd.cmd"));

        std::fs::remove_file(base_dir.join("cd.cmd")).unwrap();
        std::fs::remove_dir_all(base_dir).unwrap();
    }
}
