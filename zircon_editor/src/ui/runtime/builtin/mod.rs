#[path = "component_descriptors.rs"]
mod component_descriptors;
#[path = "material_lab_template_bindings.rs"]
mod material_lab_template_bindings;
#[path = "showcase_template_bindings.rs"]
mod showcase_template_bindings;
#[path = "bindings.rs"]
mod template_bindings;
#[path = "documents.rs"]
mod template_documents;
#[path = "unreal_editor_template_bindings.rs"]
mod unreal_editor_template_bindings;
#[path = "builtin_ui_bindings.rs"]
mod workbench_extension_module_template_bindings;
#[path = "generated_bottom_template_bindings.rs"]
mod workbench_generated_bottom_template_bindings;
#[path = "module_template_bindings.rs"]
mod workbench_module_template_bindings;
#[path = "window_template_bindings.rs"]
mod workbench_window_template_bindings;

pub(crate) use component_descriptors::builtin_component_descriptors;
pub(crate) use template_bindings::builtin_template_bindings;
pub(crate) use template_documents::{
    builtin_template_documents, ASSET_SURFACE_DOCUMENT_ID, INSPECTOR_SURFACE_DOCUMENT_ID,
    MATERIAL_COMPONENT_LAB_WINDOW_DOCUMENT_ID, PANE_ANIMATION_GRAPH_BODY_DOCUMENT_ID,
    PANE_ANIMATION_SEQUENCE_BODY_DOCUMENT_ID, PANE_BUILD_EXPORT_BODY_DOCUMENT_ID,
    PANE_CONSOLE_BODY_DOCUMENT_ID, PANE_GENERATED_BOTTOM_BODY_DOCUMENT_ID,
    PANE_HIERARCHY_BODY_DOCUMENT_ID, PANE_INSPECTOR_BODY_DOCUMENT_ID,
    PANE_MODULE_PLUGINS_BODY_DOCUMENT_ID, PANE_PERFORMANCE_TIMELINE_BODY_DOCUMENT_ID,
    PANE_RUNTIME_DIAGNOSTICS_BODY_DOCUMENT_ID, PANE_SURFACE_DOCUMENT_ID,
    SCENE_VIEWPORT_TOOLBAR_DOCUMENT_ID, UNREAL_EDITOR_SHELL_DOCUMENT_ID,
    UNREAL_EDITOR_WINDOW_DOCUMENT_ID, WELCOME_SURFACE_DOCUMENT_ID,
};
