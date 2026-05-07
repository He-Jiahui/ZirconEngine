mod collect;
mod resolve;

pub use collect::{collect_document_resource_dependencies, unique_resource_references};
pub use resolve::{validate_resource_dependency_files, UiResourcePathResolver};
