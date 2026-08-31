mod admission;
mod error;
mod limits;
mod migration;
mod parse;
mod summary;

pub use error::ProjectManifestSummaryError;
pub use limits::{
    MAX_PROJECT_ASSET_ROOTS, MAX_PROJECT_MANIFEST_ARRAY_ITEMS, MAX_PROJECT_MANIFEST_BYTES,
    MAX_PROJECT_MANIFEST_NESTING_DEPTH, MAX_PROJECT_MANIFEST_TABLE_ENTRIES,
};
pub use migration::{load_project_manifest_value_from_toml_str, PROJECT_MANIFEST_FORMAT_VERSION};
pub use parse::validate_engine_version_req;
pub use summary::ProjectManifestSummary;
