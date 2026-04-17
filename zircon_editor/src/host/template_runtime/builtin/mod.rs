mod component_descriptors;
mod template_bindings;
mod template_documents;

pub(crate) use component_descriptors::builtin_component_descriptors;
pub(crate) use template_bindings::builtin_template_bindings;
pub(crate) use template_documents::{
    builtin_template_documents, ASSET_SURFACE_DOCUMENT_ID, FLOATING_WINDOW_SOURCE_DOCUMENT_ID,
    INSPECTOR_SURFACE_DOCUMENT_ID, PANE_SURFACE_DOCUMENT_ID, SCENE_VIEWPORT_TOOLBAR_DOCUMENT_ID,
    WELCOME_SURFACE_DOCUMENT_ID, WORKBENCH_SHELL_DOCUMENT_ID,
};
