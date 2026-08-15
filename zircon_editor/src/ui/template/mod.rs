mod adapter;
mod catalog;
mod registry;
mod service;

pub use adapter::EditorTemplateAdapter;
pub use catalog::{
    EDITOR_COMPONENT_CATALOG_MANIFEST_FORMAT_VERSION, EditorComponentCatalog,
    EditorComponentCatalogManifestError, EditorComponentDescriptor, EditorComponentTier,
    EditorPropContract, EditorPropDefault, EditorPropLiteral, EditorSlotContract,
    EditorTemplateError, parse_editor_component_catalog_manifest,
};
pub use registry::EditorTemplateRegistry;
pub use service::EditorTemplateRuntimeService;
