use super::super::{
    theme_authoring::{
        build_imported_theme_local_merge_preview, build_theme_refactor_items,
        build_theme_rule_helper_items, can_prune_duplicate_local_theme_overrides,
    },
    theme_cascade_inspection::build_theme_cascade_inspection,
    theme_compare::build_theme_compare_items,
    theme_summary::{build_theme_source_details, build_theme_summary},
    ui_asset_editor_session::UiAssetEditorSession,
};
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

pub(super) struct UiAssetThemePaneData {
    pub(super) source_items: Vec<String>,
    pub(super) source_selected_index: i32,
    pub(super) selected_source_reference: String,
    pub(super) selected_source_kind: String,
    pub(super) selected_source_token_count: i32,
    pub(super) selected_source_rule_count: i32,
    pub(super) selected_source_available: bool,
    pub(super) can_promote_local: bool,
    pub(super) selected_source_token_items: Vec<String>,
    pub(super) selected_source_rule_items: Vec<String>,
    pub(super) cascade_layer_items: Vec<String>,
    pub(super) cascade_token_items: Vec<String>,
    pub(super) cascade_rule_items: Vec<String>,
    pub(super) compare_items: Vec<String>,
    pub(super) merge_preview_items: Vec<String>,
    pub(super) rule_helper_items: Vec<String>,
    pub(super) refactor_items: Vec<String>,
    pub(super) promote_asset_id: String,
    pub(super) promote_document_id: String,
    pub(super) promote_display_name: String,
    pub(super) can_edit_promote_draft: bool,
    pub(super) can_prune_duplicate_local_overrides: bool,
}

impl UiAssetEditorSession {
    pub(super) fn theme_pane_presentation(&self) -> UiAssetThemePaneData {
        zircon_runtime::profile_scope!("editor", "asset_editor.presentation", "theme",);
        let summary = build_theme_summary(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            self.selected_theme_source_key.as_deref(),
        );
        let source_details = build_theme_source_details(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            self.selected_theme_source_key.as_deref(),
        );
        let cascade = build_theme_cascade_inspection(
            &self.last_valid_document,
            &self.compiler_imports.styles,
        );
        let compare_items = build_theme_compare_items(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            self.selected_theme_source_key.as_deref(),
        );
        let rule_helper_items = build_theme_rule_helper_items(
            &self.last_valid_document,
            &self.compiler_imports.styles,
            self.selected_theme_source_key.as_deref(),
        );
        let refactor_items =
            build_theme_refactor_items(&self.last_valid_document, &self.compiler_imports.styles);
        let merge_preview_items = self
            .selected_theme_source_key
            .as_deref()
            .filter(|key| *key != "local")
            .and_then(|reference| {
                self.compiler_imports
                    .styles
                    .get(reference)
                    .map(|imported_style| {
                        build_imported_theme_local_merge_preview(
                            &self.last_valid_document,
                            reference,
                            imported_style,
                        )
                    })
            })
            .unwrap_or_default();
        let promote_draft = self.selected_promote_theme_draft();
        record_current_ui_perf_counter(UiPerfCounter::AssetEditorPaneThemeBuildCount, 1.0);
        UiAssetThemePaneData {
            source_items: summary.items,
            source_selected_index: summary.selected_index,
            selected_source_reference: summary.selected_reference,
            selected_source_kind: summary.selected_kind.clone(),
            selected_source_token_count: summary.selected_token_count,
            selected_source_rule_count: summary.selected_rule_count,
            selected_source_available: summary.selected_available,
            can_promote_local: summary.can_promote_local,
            selected_source_token_items: source_details.token_items,
            selected_source_rule_items: source_details.rule_items,
            cascade_layer_items: cascade.layer_items,
            cascade_token_items: cascade.token_items,
            cascade_rule_items: cascade.rule_items,
            compare_items,
            merge_preview_items,
            rule_helper_items,
            refactor_items,
            promote_asset_id: promote_draft
                .as_ref()
                .map(|draft| draft.asset_id.clone())
                .unwrap_or_default(),
            promote_document_id: promote_draft
                .as_ref()
                .map(|draft| draft.document_id.clone())
                .unwrap_or_default(),
            promote_display_name: promote_draft
                .as_ref()
                .map(|draft| draft.display_name.clone())
                .unwrap_or_default(),
            can_edit_promote_draft: summary.selected_kind == "Local" && summary.can_promote_local,
            can_prune_duplicate_local_overrides: can_prune_duplicate_local_theme_overrides(
                &self.last_valid_document,
                &self.compiler_imports.styles,
            ),
        }
    }
}
