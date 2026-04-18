mod adapter;
mod catalog;
mod registry;

pub use adapter::EditorTemplateAdapter;
pub use catalog::{EditorComponentCatalog, EditorComponentDescriptor, EditorTemplateError};
pub use registry::EditorTemplateRegistry;
