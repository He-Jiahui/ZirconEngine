#[path = "manifest/error.rs"]
mod error;
#[path = "manifest/export_profiles.rs"]
mod export_profiles;
#[path = "manifest/load.rs"]
mod load;
#[path = "manifest/project_manifest.rs"]
mod project_manifest;
#[path = "manifest/save.rs"]
mod save;
#[path = "manifest/validation.rs"]
mod validation;

pub use error::ProjectManifestError;
pub use project_manifest::ProjectManifest;
