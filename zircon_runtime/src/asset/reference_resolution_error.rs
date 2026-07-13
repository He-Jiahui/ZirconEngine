use std::path::PathBuf;

use thiserror::Error;
use zircon_runtime_interface::project::{AssetRefError, RelPathError};
use zircon_runtime_interface::resource::{AssetUuid, ResourceLocator};

#[derive(Debug, Error)]
pub enum ReferenceResolutionError {
    #[error("persisted asset reference has no payload")]
    MissingPayload,
    #[error("unsupported persisted asset reference scheme in {locator}")]
    UnsupportedScheme { locator: ResourceLocator },
    #[error("authoring asset {path} requires a project registry resolver")]
    ProjectContextRequired { path: PathBuf },
    #[error("asset guid {guid} is not registered")]
    MissingGuid { guid: AssetUuid },
    #[error("asset path {path} is not registered or does not resolve")]
    MissingPath { path: String },
    #[error("asset path {path} resolves in multiple project roots")]
    AmbiguousPath { path: String },
    #[error("failed to inspect project asset path {path}: {source}")]
    PathIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("asset reference {guid} and path hint {path} are both dangling")]
    Dangling { guid: AssetUuid, path: String },
    #[error("asset reference guid {guid} and path hint {path} resolve to different entries")]
    Conflict { guid: AssetUuid, path: String },
    #[error("asset registry disagrees with persisted reference: {message}")]
    Registry { message: String },
    #[error("asset reference path {path} is invalid: {source}")]
    Path {
        path: String,
        #[source]
        source: RelPathError,
    },
    #[error("asset reference contract is invalid: {source}")]
    AssetRef {
        #[source]
        source: AssetRefError,
    },
}

#[cfg(test)]
mod tests {
    use std::error::Error as _;

    use super::*;

    #[test]
    fn path_io_preserves_the_operating_system_error_source() {
        let error = ReferenceResolutionError::PathIo {
            path: PathBuf::from("assets/blocked.glb"),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "blocked"),
        };
        let source = error.source().expect("path io keeps source chain");
        assert!(source.to_string().contains("blocked"));
    }
}
