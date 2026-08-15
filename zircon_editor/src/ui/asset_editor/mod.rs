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
    UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID, UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_DOCUMENT_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID, UI_ASSET_EDITOR_WINDOW_ID, UiAssetEditorMode,
    UiAssetEditorReflectionModel, UiAssetEditorRoute, UiAssetEditorShellState,
    UiAssetPreviewPreset, UiDesignerPreviewInteractDispatch, UiDesignerSelectionModel,
    UiDesignerToolMode, UiMatchedStyleRuleReflection, UiStyleInspectorReflectionModel,
    ui_asset_editor_window_descriptor,
};
pub(crate) use contract::{
    UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_PATH, UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_PATH,
};
pub use diagnostics::{UiAssetEditorDiagnostic, UiAssetEditorDiagnosticSeverity};
#[cfg(test)]
pub(crate) use node_projection::ui_asset_editor_surface_for_test;
pub(crate) use node_projection::{
    apply_ui_asset_editor_designer_tool_mode, ui_asset_editor_node_projection,
};
pub use presentation::{
    UiAssetEditorPanePresentation, UiAssetEditorPreviewCanvasNode,
    UiAssetEditorPreviewCanvasSlotTarget, UiAssetEditorWidgetPropStateItem,
};
pub use preview::UiAssetPreviewHost;
pub use replay_workspace::{UiAssetEditorReplayWorkspace, UiAssetEditorReplayWorkspaceResult};
pub use session::{
    UI_ASSET_EDITOR_BUG_REPORT_REPLAY_ARTIFACT_SCHEMA_VERSION,
    UI_ASSET_EDITOR_COMMAND_JOURNAL_SCHEMA_VERSION, UiAssetEditorBugReportReplayArtifact,
    UiAssetEditorCommandJournal, UiAssetEditorCommandJournalEntry,
    UiAssetEditorCommandJournalReplayError, UiAssetEditorCommandJournalReplayReport,
    UiAssetEditorJournalCommand, UiAssetEditorReplayArtifactRecord,
    UiAssetEditorReplayArtifactRoute, UiAssetEditorReplayCommandSummary,
    UiAssetEditorReplayExternalEffectSummary, UiAssetEditorReplayResult,
    UiAssetEditorReplaySelectionSummary, UiAssetEditorReplaySourceSummary, UiAssetEditorSession,
    UiAssetEditorSessionError,
};
pub use source::UiAssetSourceBuffer;
pub use tree::UiAssetDragDropPolicy;
pub use undo_stack::{
    UiAssetEditorExternalEffect, UiAssetEditorSourceCursorSnapshot,
    UiAssetEditorUndoExternalEffects, UiAssetEditorUndoReplayRecord, UiAssetEditorUndoStack,
    UiAssetEditorUndoStackReplayRecord, UiAssetEditorUndoTransition,
    apply_external_effects_to_asset_sources,
};
