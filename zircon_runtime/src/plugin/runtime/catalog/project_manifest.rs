#[path = "project_manifest/completion.rs"]
mod completion;
#[path = "project_manifest/selection_defaults.rs"]
mod selection_defaults;

pub(super) use completion::{catalog_project_manifest, complete_project_manifest};
