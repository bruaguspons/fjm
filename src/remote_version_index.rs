//! Shared contract for per-tool remote version indices.
//!
//! Adoptium (JSON, os/arch-aware, embedded checksums) and Maven Central
//! (XML, no os/arch axis, `.sha512` sidecar checksums) have incompatible
//! wire shapes, so this trait documents the minimal contract a remote index
//! module must satisfy, without forcing either implementation to share code
//! it doesn't have. A future `remote_gradle_index.rs` implements the same
//! trait; no change to this file is required to add it.

use crate::config::FjmConfig;
use crate::downloader::ChecksumSource;
use crate::version::Version;
use url::Url;

/// A resolved, installable asset: a download link plus enough information
/// to verify and extract it once downloaded.
#[derive(Debug, Clone)]
pub struct ResolvedAsset {
    pub version: Version,
    pub link: Url,
    pub checksum: ChecksumSource,
    pub name: String,
}

#[allow(
    dead_code,
    reason = "Documents the shared per-tool remote-index contract (see design.md); the Java CLI flow uses remote_node_index's richer LTS/exact-patch-aware free functions directly, and the Maven CLI flow currently calls remote_maven_index's free functions directly too, so no caller constructs a `dyn`/generic RemoteVersionIndex today. Kept type-checked via AdoptiumIndex/MavenIndex so a future remote_gradle_index.rs has a compiler-verified contract to match."
)]
pub trait RemoteVersionIndex {
    type Error;

    /// Lists every version this index currently exposes.
    fn list_remote(&self, config: &FjmConfig) -> Result<Vec<Version>, Self::Error>;

    /// Resolves `requested` into a downloadable, checksum-verifiable asset.
    fn resolve_asset(
        &self,
        config: &FjmConfig,
        requested: &Version,
    ) -> Result<ResolvedAsset, Self::Error>;
}
