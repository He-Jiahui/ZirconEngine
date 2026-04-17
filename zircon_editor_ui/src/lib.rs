mod activity;
mod binding;
mod control;
mod reflection;
mod template;
mod ui_asset_editor;

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
    UiMatchedStyleRuleReflection, UiStyleInspectorReflectionModel, UI_ASSET_EDITOR_WINDOW_ID,
};

#[cfg(test)]
mod tests;
