#[path = "binding/mod.rs"]
mod binding;
#[path = "command.rs"]
mod command;
#[path = "contract.rs"]
mod contract;
#[path = "diagnostics/mod.rs"]
mod diagnostics;
#[path = "document_diff.rs"]
mod document_diff;
#[path = "node_projection.rs"]
mod node_projection;
#[path = "palette/mod.rs"]
pub(crate) mod palette;
#[path = "palette_target_chooser.rs"]
mod palette_target_chooser;
#[path = "presentation.rs"]
mod presentation;
#[path = "preview/mod.rs"]
pub(crate) mod preview;
#[path = "promote_widget.rs"]
mod promote_widget;
#[path = "replay_workspace.rs"]
mod replay_workspace;
#[path = "session/mod.rs"]
mod session;

pub(crate) fn project_authoring_document_to_v2(
    document: &zircon_runtime_interface::ui::template::UiAssetDocument,
) -> Result<zircon_runtime_interface::ui::v2::UiV2AssetDocument, UiAssetEditorSessionError> {
    session::lifecycle::v2_projection::legacy_projection_document_to_v2_document(document, None)
}

pub(crate) fn project_v2_document_to_authoring(
    document: &zircon_runtime_interface::ui::v2::UiV2AssetDocument,
) -> Result<zircon_runtime_interface::ui::template::UiAssetDocument, UiAssetEditorSessionError> {
    session::lifecycle::v2_projection::v2_document_to_legacy_projection_document(document)
}

pub(crate) fn serialize_authoring_document_as_v2(
    document: &zircon_runtime_interface::ui::template::UiAssetDocument,
) -> Result<String, UiAssetEditorSessionError> {
    session::lifecycle::v2_projection::serialize_v2_projection_document(document, None)
}
#[path = "source/mod.rs"]
mod source;
#[path = "style/mod.rs"]
pub(crate) mod style;
#[path = "tree/mod.rs"]
pub(crate) mod tree;
#[path = "undo_stack.rs"]
mod undo_stack;
#[path = "value_path.rs"]
pub(crate) mod value_path;

#[cfg(test)]
pub use command::UiAssetEditorInverseTreeEdit;
pub use command::{
    UiAssetEditorCommand, UiAssetEditorDocumentReplayBundle, UiAssetEditorDocumentReplayCommand,
    UiAssetEditorTreeEdit, UiAssetEditorTreeEditKind,
};
pub use contract::{
    ui_asset_editor_window_descriptor, UiAssetEditorMode, UiAssetEditorReflectionModel,
    UiAssetEditorRoute, UiAssetEditorShellState, UiAssetPreviewPreset,
    UiDesignerPreviewInteractDispatch, UiDesignerSelectionModel, UiDesignerToolMode,
    UiMatchedStyleRuleReflection, UiStyleInspectorReflectionModel,
    UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID, UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_DOCUMENT_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID, UI_ASSET_EDITOR_WINDOW_ID,
};
pub(crate) use contract::{
    UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_PATH, UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_PATH,
};
pub use diagnostics::{UiAssetEditorDiagnostic, UiAssetEditorDiagnosticSeverity};
#[cfg(test)]
pub(crate) use node_projection::ui_asset_editor_surface_for_test;
pub(crate) use node_projection::{
    apply_ui_asset_editor_designer_tool_mode, ui_asset_editor_node_projection,
    ui_asset_editor_node_projection_with_palette,
};
pub use presentation::{
    UiAssetEditorPanePresentation, UiAssetEditorPreviewCanvasNode,
    UiAssetEditorPreviewCanvasSlotTarget, UiAssetEditorWidgetPropStateItem,
};
pub use preview::UiAssetPreviewHost;
pub use replay_workspace::{UiAssetEditorReplayWorkspace, UiAssetEditorReplayWorkspaceResult};
pub use session::{
    UiAssetEditorBugReportReplayArtifact, UiAssetEditorCommandJournal,
    UiAssetEditorCommandJournalEntry, UiAssetEditorCommandJournalReplayError,
    UiAssetEditorCommandJournalReplayReport, UiAssetEditorJournalCommand,
    UiAssetEditorReplayArtifactRecord, UiAssetEditorReplayArtifactRoute,
    UiAssetEditorReplayCommandSummary, UiAssetEditorReplayExternalEffectSummary,
    UiAssetEditorReplayResult, UiAssetEditorReplaySelectionSummary,
    UiAssetEditorReplaySourceSummary, UiAssetEditorSession, UiAssetEditorSessionError,
    UI_ASSET_EDITOR_BUG_REPORT_REPLAY_ARTIFACT_SCHEMA_VERSION,
    UI_ASSET_EDITOR_COMMAND_JOURNAL_SCHEMA_VERSION,
};
pub use source::UiAssetSourceBuffer;
pub use tree::UiAssetDragDropPolicy;
pub use undo_stack::{
    apply_external_effects_to_asset_sources, UiAssetEditorExternalEffect,
    UiAssetEditorSourceCursorSnapshot, UiAssetEditorUndoExternalEffects,
    UiAssetEditorUndoReplayRecord, UiAssetEditorUndoStack, UiAssetEditorUndoStackReplayRecord,
    UiAssetEditorUndoTransition,
};
