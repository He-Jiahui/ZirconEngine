mod adapter;
mod catalog;
mod registry;
mod service;

pub use adapter::EditorTemplateAdapter;
pub use catalog::{EditorComponentCatalog, EditorComponentDescriptor, EditorTemplateError};
pub use registry::EditorTemplateRegistry;
pub use service::EditorTemplateRuntimeService;
