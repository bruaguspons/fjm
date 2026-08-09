use crate::config::FjmConfig;
use crate::default_version;
use crate::tool_kind::ToolKind;
use crate::user_version::UserVersion;
use crate::version_file_strategy::VersionFileStrategy;
use encoding_rs_io::DecodeReaderBytes;
use log::info;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

#[allow(
    dead_code,
    reason = "ToolKind::Java-default convenience wrapper around get_user_version_for_directory_for_tool; kept for symmetry with the rest of the tool-scoped API"
)]
pub fn get_user_version_for_directory(
    path: impl AsRef<Path>,
    config: &FjmConfig,
) -> Option<UserVersion> {
    get_user_version_for_directory_for_tool(path, config, ToolKind::Java)
}

pub fn get_user_version_for_directory_for_tool(
    path: impl AsRef<Path>,
    config: &FjmConfig,
    tool: ToolKind,
) -> Option<UserVersion> {
    match config.version_file_strategy() {
        VersionFileStrategy::Local => get_user_version_for_single_directory(path, config, tool),
        VersionFileStrategy::Recursive => {
            get_user_version_for_directory_recursive(path, config, tool).or_else(|| {
                info!("Did not find anything recursively. Falling back to default alias.");
                default_version::find_default_version_for(config, tool).map(UserVersion::Full)
            })
        }
    }
}

fn get_user_version_for_directory_recursive(
    path: impl AsRef<Path>,
    config: &FjmConfig,
    tool: ToolKind,
) -> Option<UserVersion> {
    let mut current_path = Some(path.as_ref());

    while let Some(child_path) = current_path {
        if let Some(version) = get_user_version_for_single_directory(child_path, config, tool) {
            return Some(version);
        }

        current_path = child_path.parent();
    }

    None
}

fn get_user_version_for_single_directory(
    path: impl AsRef<Path>,
    config: &FjmConfig,
    tool: ToolKind,
) -> Option<UserVersion> {
    let path = path.as_ref();

    let new_path = path.join(tool.version_file_name());
    info!(
        "Looking for version file in {}. exists? {}",
        new_path.display(),
        new_path.exists()
    );
    get_user_version_for_file(&new_path, config)
}

pub fn get_user_version_for_file(
    path: impl AsRef<Path>,
    _config: &FjmConfig,
) -> Option<UserVersion> {
    let file = std::fs::File::open(path).ok()?;
    let file = {
        let mut reader = DecodeReaderBytes::new(file);
        let mut version = String::new();
        reader.read_to_string(&mut version).map(|_| version)
    };

    match file {
        Err(err) => {
            info!("Can't read file: {err}");
            None
        }
        Ok(version) => {
            info!("Found string {version:?} in version file");
            UserVersion::from_str(version.trim()).ok()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_and_maven_have_distinct_version_file_names() {
        assert_eq!(ToolKind::Java.version_file_name(), ".java-version");
        assert_eq!(ToolKind::Maven.version_file_name(), ".maven-version");
    }

    #[test]
    fn test_no_nvmrc_or_package_json_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let config = FjmConfig::default();

        std::fs::write(dir.path().join(".nvmrc"), "16.0.0").unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"engines":{"node":"16"}}"#,
        )
        .unwrap();

        assert!(
            get_user_version_for_single_directory(dir.path(), &config, ToolKind::Java).is_none()
        );
    }

    #[test]
    fn test_java_version_file_is_read() {
        let dir = tempfile::tempdir().unwrap();
        let config = FjmConfig::default();

        std::fs::write(dir.path().join(".java-version"), "17.0.2").unwrap();

        assert!(
            get_user_version_for_single_directory(dir.path(), &config, ToolKind::Java).is_some()
        );
    }

    #[test]
    fn test_maven_version_file_is_read_independently_of_java() {
        let dir = tempfile::tempdir().unwrap();
        let config = FjmConfig::default();

        std::fs::write(dir.path().join(".java-version"), "17").unwrap();
        std::fs::write(dir.path().join(".maven-version"), "3.9.9").unwrap();

        let java = get_user_version_for_single_directory(dir.path(), &config, ToolKind::Java);
        let maven = get_user_version_for_single_directory(dir.path(), &config, ToolKind::Maven);

        assert!(java.is_some());
        assert!(maven.is_some());
        assert_eq!(java.unwrap().to_string(), "v17.x.x");
        assert_eq!(maven.unwrap().to_string(), "v3.9.9");
    }

    #[test]
    fn test_maven_version_file_absent_does_not_leak_java_version() {
        let dir = tempfile::tempdir().unwrap();
        let config = FjmConfig::default();

        std::fs::write(dir.path().join(".java-version"), "17").unwrap();

        assert!(
            get_user_version_for_single_directory(dir.path(), &config, ToolKind::Maven).is_none()
        );
    }
}
