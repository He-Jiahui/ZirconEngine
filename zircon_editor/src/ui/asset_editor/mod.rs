mod binding;
mod command;
mod contract;
mod diagnostics;
mod document_diff;
mod node_projection;
pub(crate) mod palette;
mod palette_target_chooser;
mod presentation;
pub(crate) mod preview;
mod promote_widget;
mod replay_workspace;
mod session;
mod source;
pub(crate) mod style;
pub(crate) mod tree;
mod undo_stack;
pub(crate) mod value_path;

#[cfg(test)]
pub use command::UiAssetEditorInverseTreeEdit;
pub use command::{
    UiAssetEditorCommand, UiAssetEditorDocumentReplayBundle, UiAssetEditorDocumentReplayCommand,
    UiAssetEditorTreeEdit, UiAssetEditorTreeEditKind,
};
pub use contract::{
    ui_asset_editor_window_descriptor, UiAssetEditorMode, UiAssetEditorReflectionModel,
    UiAssetEditorRoute, UiAssetPreviewPreset, UiDesignerSelectionModel,
    UiMatchedStyleRuleReflection, UiStyleInspectorReflectionModel,
    UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID, UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_DOCUMENT_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID, UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_ASSET_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_BUTTON_REFERENCE,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_HEADER_SHELL_REFERENCE,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE, UI_ASSET_EDITOR_WINDOW_ID,
};
pub use diagnostics::{UiAssetEditorDiagnostic, UiAssetEditorDiagnosticSeverity};
pub(crate) use node_projection::ui_asset_editor_node_projection;
pub use presentation::{
    UiAssetEditorPanePresentation, UiAssetEditorPreviewCanvasNode,
    UiAssetEditorPreviewCanvasSlotTarget,
};
pub use preview::UiAssetPreviewHost;
pub use replay_workspace::{UiAssetEditorReplayWorkspace, UiAssetEditorReplayWorkspaceResult};
pub use session::{UiAssetEditorReplayResult, UiAssetEditorSession, UiAssetEditorSessionError};
pub use source::UiAssetSourceBuffer;
pub use tree::UiAssetDragDropPolicy;
pub use undo_stack::{
    apply_external_effects_to_asset_sources, UiAssetEditorExternalEffect,
    UiAssetEditorSourceCursorSnapshot, UiAssetEditorUndoExternalEffects,
    UiAssetEditorUndoReplayRecord, UiAssetEditorUndoStack, UiAssetEditorUndoTransition,
};
