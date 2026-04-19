//! Editor-only UI contracts, Slint host runtime, and workbench projection.

mod activity;
pub(crate) mod binding;
pub(crate) mod binding_dispatch;
mod control;
mod reflection;
pub(crate) mod slint_host;
pub(crate) mod template;
pub(crate) mod template_runtime;
mod ui_asset_editor;
pub(crate) mod workbench;

pub use activity::{
    ActivityDrawerSlotPreference, ActivityViewDescriptor, ActivityWindowDescriptor,
};
pub use binding::{
    inspector_field_control_id, AssetCommand, DockCommand, DraftCommand, EditorUiBinding,
    EditorUiBindingPayload, EditorUiEventKind, EditorUiRouter, InspectorFieldChange,
    SelectionCommand, ViewportCommand, WelcomeCommand,
};
pub use control::{EditorUiControlService, EditorUiError};
pub use reflection::{
    EditorActivityHost, EditorActivityKind, EditorActivityReflection, EditorDrawerReflectionModel,
    EditorFloatingWindowReflectionModel, EditorHostPageReflectionModel,
    EditorMenuItemReflectionModel, EditorUiReflectionAdapter, EditorWorkbenchReflectionModel,
};
pub use template::{
    EditorComponentCatalog, EditorComponentDescriptor, EditorTemplateAdapter, EditorTemplateError,
    EditorTemplateRegistry,
};
pub use ui_asset_editor::{
    ui_asset_editor_window_descriptor, UiAssetEditorMode, UiAssetEditorReflectionModel,
    UiAssetEditorRoute, UiAssetPreviewPreset, UiDesignerSelectionModel,
    UiMatchedStyleRuleReflection, UiStyleInspectorReflectionModel,
    UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID, UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_DOCUMENT_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID, UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_ASSET_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_BUTTON_REFERENCE,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE, UI_ASSET_EDITOR_WINDOW_ID,
};
