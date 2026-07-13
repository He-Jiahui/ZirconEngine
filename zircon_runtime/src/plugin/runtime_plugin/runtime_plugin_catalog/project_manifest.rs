mod completion;
mod lookup;
mod selection_defaults;

pub(super) use completion::{catalog_project_manifest, complete_project_manifest};
pub(super) use lookup::project_selection_for_package;
