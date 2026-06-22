//! High-frequency runtime UI imports for surface, template, and UI asset code.

pub use super::component::{
    apply_component_event, inspector_selected_entity_data_source, validate_component_descriptor,
    UiComponentDescriptorError, UiComponentDescriptorRegistry, UiComponentPaletteEntry,
    UiComponentStateRuntimeExt,
};
pub use super::event_ui::UiEventManager;
pub use super::layout::{
    compute_layout_tree, compute_virtual_list_window, taffy_display_for_family,
    taffy_style_for_container, ui_layout_pass_stage_names, virtual_window_for_scrollable_box,
    UiLayoutPassStage, UI_LAYOUT_PASS_ORDER,
};
pub use super::module::{
    module_descriptor, UiConfig, UiModule, UiRuntimeDriver, UI_EVENT_MANAGER_NAME, UI_MODULE_NAME,
    UI_RUNTIME_DRIVER_NAME,
};
pub use super::style::{
    resolve_button_style_from_values, resolve_property, ButtonStyleFields, ElementStyleFields,
    SharedStyleSheetScope, StyleField, StyleProperty, StyleSheetScope,
};
pub use super::surface::{
    extract_ui_render_tree, extract_ui_render_tree_from_arranged, hit_test_surface_frame,
    hit_test_surface_frame_with_query, UiDebugTimelineStore, UiPropertyMutationReport,
    UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface, UiSurfaceComponentStateStore,
    UiSurfaceInputState, UiSurfaceNodePool, UiSurfaceNodePoolReport, UiSurfaceRebuildReport,
};
pub use super::template::{
    UiAssetCompileCache, UiAssetLoader, UiCompiledDocument, UiDocumentCompiler, UiTemplateInstance,
    UiTemplateLoader, UiTemplateRuntimePipeline, UiTemplateRuntimePipelineError,
    UiTemplateSurfaceBuilder, UiTemplateTreeBuilder, UI_TEMPLATE_RUNTIME_PIPELINE_STAGES,
};
pub use super::v2::{
    UiV2AssetLoader, UiV2CompiledDocument, UiV2DocumentCompiler, UiV2PrototypeStore,
    UiV2PrototypeStoreBuilder, UiV2StyleResolver, UiV2SurfaceBuilder, UiZuiAssetLoader,
};
pub use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiNodePath, UiTreeId};
