#[path = "adapter.rs"]
mod adapter;
#[path = "catalog.rs"]
mod catalog;
#[path = "registry.rs"]
mod registry;
#[path = "service.rs"]
mod service;

pub use adapter::EditorTemplateAdapter;
pub use catalog::{
    parse_editor_component_catalog_manifest, EditorComponentCatalog,
    EditorComponentCatalogManifestError, EditorComponentDescriptor, EditorComponentTier,
    EditorPropContract, EditorPropDefault, EditorPropLiteral, EditorSlotContract,
    EditorTemplateError, EDITOR_COMPONENT_CATALOG_MANIFEST_FORMAT_VERSION,
};
pub use registry::EditorTemplateRegistry;
pub use service::EditorTemplateRuntimeService;
