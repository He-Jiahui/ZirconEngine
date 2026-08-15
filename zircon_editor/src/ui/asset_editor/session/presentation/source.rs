use super::super::{
    source_sync::build_source_selection_summary, ui_asset_editor_session::UiAssetEditorSession,
};

pub(super) struct UiAssetSourcePaneData {
    pub(super) selected_block_label: String,
    pub(super) selected_line: i32,
    pub(super) selected_excerpt: String,
    pub(super) roundtrip_status: String,
    pub(super) outline_items: Vec<String>,
    pub(super) outline_selected_index: i32,
    pub(super) structured_diagnostic_items: Vec<String>,
}

impl UiAssetEditorSession {
    pub(super) fn source_pane_presentation(&self) -> UiAssetSourcePaneData {
        zircon_runtime::profile_scope!("editor", "asset_editor.presentation", "source",);
        let source_outline = self.roundtrip_source_outline_index();
        let source_summary = build_source_selection_summary(
            &source_outline,
            &self.selection,
            &self.diagnostics,
            self.selected_source_line_offset(),
        );
        let outline_selected_index = self
            .selection
            .primary_node_id
            .as_deref()
            .or_else(|| {
                self.structured_diagnostics
                    .iter()
                    .find_map(|diagnostic| diagnostic.target_node_id.as_deref())
            })
            .and_then(|node_id| source_outline.index_for_node(node_id))
            .map(|index| index as i32)
            .unwrap_or(-1);
        UiAssetSourcePaneData {
            selected_block_label: source_summary.block_label,
            selected_line: source_summary.line,
            selected_excerpt: source_summary.excerpt,
            roundtrip_status: source_summary.roundtrip_status,
            outline_items: source_outline
                .entries()
                .iter()
                .map(|entry| format!("line {} • {}", entry.line, entry.block_label))
                .collect(),
            outline_selected_index,
            structured_diagnostic_items: self
                .structured_diagnostics
                .iter()
                .map(|diagnostic| {
                    format!(
                        "{} [{}] {}: {}",
                        diagnostic.severity.as_str(),
                        diagnostic.code,
                        diagnostic.source_path,
                        diagnostic.message
                    )
                })
                .collect(),
        }
    }
}
