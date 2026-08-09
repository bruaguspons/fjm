use crate::tool_kind::ToolKind;
use crate::version::Version;
use std::path::Path;
use thiserror::Error;

#[allow(
    dead_code,
    reason = "ToolKind::Java-default convenience wrapper around list_for_tool; kept for symmetry with the rest of the tool-scoped API"
)]
pub fn list<P: AsRef<Path>>(installations_dir: P) -> Result<Vec<Version>, Error> {
    list_for_tool(installations_dir, ToolKind::Java)
}

pub fn list_for_tool<P: AsRef<Path>>(
    installations_dir: P,
    tool: ToolKind,
) -> Result<Vec<Version>, Error> {
    let mut vec = vec![];
    for result_entry in installations_dir.as_ref().read_dir()? {
        let entry = result_entry?;
        if entry
            .file_name()
            .to_str()
            .is_some_and(|s| s.starts_with('.'))
        {
            continue;
        }

        let path = entry.path();
        let filename = path
            .file_name()
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))?
            .to_str()
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))?;
        let version = Version::parse(filename, tool)?;
        vec.push(version);
    }
    Ok(vec)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IoError {
        #[from]
        source: std::io::Error,
    },
    #[error(transparent)]
    VersionError {
        #[from]
        source: crate::version::Error,
    },
}
