use crate::config::FjmConfig;
use crate::default_version;
use crate::user_version::UserVersion;
use crate::version_file_strategy::VersionFileStrategy;
use encoding_rs_io::DecodeReaderBytes;
use log::info;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

const PATH_PARTS: [&str; 1] = [".java-version"];

pub fn get_user_version_for_directory(
    path: impl AsRef<Path>,
    config: &FjmConfig,
) -> Option<UserVersion> {
    match config.version_file_strategy() {
        VersionFileStrategy::Local => get_user_version_for_single_directory(path, config),
        VersionFileStrategy::Recursive => get_user_version_for_directory_recursive(path, config)
            .or_else(|| {
                info!("Did not find anything recursively. Falling back to default alias.");
                default_version::find_default_version(config).map(UserVersion::Full)
            }),
    }
}

fn get_user_version_for_directory_recursive(
    path: impl AsRef<Path>,
    config: &FjmConfig,
) -> Option<UserVersion> {
    let mut current_path = Some(path.as_ref());

    while let Some(child_path) = current_path {
        if let Some(version) = get_user_version_for_single_directory(child_path, config) {
            return Some(version);
        }

        current_path = child_path.parent();
    }

    None
}

fn get_user_version_for_single_directory(
    path: impl AsRef<Path>,
    config: &FjmConfig,
) -> Option<UserVersion> {
    let path = path.as_ref();

    for path_part in &PATH_PARTS {
        let new_path = path.join(path_part);
        info!(
            "Looking for version file in {}. exists? {}",
            new_path.display(),
            new_path.exists()
        );
        if let Some(version) = get_user_version_for_file(&new_path, config) {
            return Some(version);
        }
    }

    None
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
    fn test_only_java_version_file_is_discovered() {
        assert_eq!(PATH_PARTS, [".java-version"]);
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

        assert!(get_user_version_for_single_directory(dir.path(), &config).is_none());
    }

    #[test]
    fn test_java_version_file_is_read() {
        let dir = tempfile::tempdir().unwrap();
        let config = FjmConfig::default();

        std::fs::write(dir.path().join(".java-version"), "17.0.2").unwrap();

        assert!(get_user_version_for_single_directory(dir.path(), &config).is_some());
    }
}
