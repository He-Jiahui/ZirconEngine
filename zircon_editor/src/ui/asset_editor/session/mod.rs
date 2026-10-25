use super::{
    binding::binding_inspector,
    command, diagnostics, palette, palette_target_chooser,
    preview::{preview_host, preview_mock, preview_projection},
    promote_widget,
    source::{source_buffer, source_sync},
    style::{
        inspector_fields, inspector_semantics, style_rule_declarations, theme_authoring,
        theme_cascade_inspection, theme_compare, theme_summary,
    },
    tree::{palette_drop, tree_editing},
    undo_stack,
};

mod binding_source_navigation;
#[path = "binding_state.rs"]
pub(crate) mod binding_state;
#[path = "canvas_anchor_state.rs"]
mod canvas_anchor_state;
#[path = "command_entry.rs"]
pub(crate) mod command_entry;
#[path = "designer_state.rs"]
pub(crate) mod designer_state;
#[path = "emergency_state.rs"]
pub(crate) mod emergency_state;
#[path = "hierarchy_projection.rs"]
pub(crate) mod hierarchy_projection;
#[path = "import_reference_access.rs"]
mod import_reference_access;
#[path = "journal.rs"]
pub(crate) mod journal;
#[path = "lifecycle.rs"]
pub(crate) mod lifecycle;
#[path = "navigation_state.rs"]
pub(crate) mod navigation_state;
mod palette_search_state;
#[path = "palette_state.rs"]
pub(crate) mod palette_state;
#[path = "presentation/mod.rs"]
pub(crate) mod presentation;
#[path = "preview_compile.rs"]
pub(crate) mod preview_compile;
#[path = "preview_state.rs"]
pub(crate) mod preview_state;
#[path = "preview_visibility_state.rs"]
mod preview_visibility_state;
#[path = "promotion_state.rs"]
pub(crate) mod promotion_state;
#[path = "replay_artifact.rs"]
pub(crate) mod replay_artifact;
#[path = "resolver_state.rs"]
pub(crate) mod resolver_state;
#[path = "root_class_policy_state.rs"]
pub(crate) mod root_class_policy_state;
#[path = "runtime_report_state.rs"]
pub(crate) mod runtime_report_state;
#[path = "session_state.rs"]
pub(crate) mod session_state;
#[path = "style_inspection.rs"]
pub(crate) mod style_inspection;
#[path = "style_rule_identity.rs"]
pub(crate) mod style_rule_identity;
#[path = "style_state.rs"]
pub(crate) mod style_state;
#[path = "theme_state.rs"]
pub(crate) mod theme_state;
#[path = "ui_asset_editor_session.rs"]
pub(crate) mod ui_asset_editor_session;
#[path = "v2_authoring.rs"]
pub(crate) mod v2_authoring;

pub use journal::{
    UiAssetEditorCommandJournal, UiAssetEditorCommandJournalEntry,
    UiAssetEditorCommandJournalReplayError, UiAssetEditorCommandJournalReplayReport,
    UiAssetEditorJournalCommand, UI_ASSET_EDITOR_COMMAND_JOURNAL_SCHEMA_VERSION,
};
pub use replay_artifact::{
    UiAssetEditorBugReportReplayArtifact, UiAssetEditorReplayArtifactRecord,
    UiAssetEditorReplayArtifactRoute, UiAssetEditorReplayCommandSummary,
    UiAssetEditorReplayExternalEffectSummary, UiAssetEditorReplaySelectionSummary,
    UiAssetEditorReplaySourceSummary, UI_ASSET_EDITOR_BUG_REPORT_REPLAY_ARTIFACT_SCHEMA_VERSION,
};
pub use ui_asset_editor_session::{
    UiAssetEditorReplayResult, UiAssetEditorSession, UiAssetEditorSessionError,
};

pub(crate) use binding_source_navigation::UiAssetBindingSourceLocation;
pub(crate) use canvas_anchor_state::{
    UiAssetCanvasAnchorPresetCommit, UiAssetCanvasAnchorPresetTicket,
};
