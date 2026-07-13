use thiserror::Error;

use zircon_runtime_interface::project::ProjectManifestSummaryError;

/// Typed project-manifest load, migration, validation, or persistence failure.
#[derive(Debug, Error)]
pub enum ProjectManifestError {
    #[error("read project manifest failed: {source}")]
    Read {
        #[source]
        source: std::io::Error,
    },
    #[error("write project manifest failed: {source}")]
    Write {
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Summary(#[from] ProjectManifestSummaryError),
    #[error("project manifest JSON value could not decode into the current schema: {source}")]
    Decode {
        #[source]
        source: serde_json::Error,
    },
    #[error("project manifest must declare at least one asset root")]
    EmptyAssetRoots,
    #[error("project manifest declares duplicate normalized asset root {root}")]
    DuplicateAssetRoot { root: String },
    #[error("project manifest asset roots {ancestor} and {descendant} overlap")]
    OverlappingAssetRoots {
        ancestor: String,
        descendant: String,
    },
    #[error("encode project manifest TOML failed: {source}")]
    Encode {
        #[source]
        source: toml::ser::Error,
    },
}
