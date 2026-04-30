mod component_descriptors;
mod showcase_template_bindings;
mod template_bindings;
mod template_documents;

pub(crate) use component_descriptors::builtin_component_descriptors;
pub(crate) use template_bindings::builtin_template_bindings;
pub(crate) use template_documents::{
    builtin_template_documents, ASSET_SURFACE_DOCUMENT_ID, INSPECTOR_SURFACE_DOCUMENT_ID,
    PANE_ANIMATION_GRAPH_BODY_DOCUMENT_ID, PANE_ANIMATION_SEQUENCE_BODY_DOCUMENT_ID,
    PANE_CONSOLE_BODY_DOCUMENT_ID, PANE_HIERARCHY_BODY_DOCUMENT_ID,
    PANE_INSPECTOR_BODY_DOCUMENT_ID, PANE_MODULE_PLUGINS_BODY_DOCUMENT_ID,
    PANE_RUNTIME_DIAGNOSTICS_BODY_DOCUMENT_ID, PANE_SURFACE_DOCUMENT_ID,
    SCENE_VIEWPORT_TOOLBAR_DOCUMENT_ID, UI_HOST_WINDOW_DOCUMENT_ID, WELCOME_SURFACE_DOCUMENT_ID,
};
