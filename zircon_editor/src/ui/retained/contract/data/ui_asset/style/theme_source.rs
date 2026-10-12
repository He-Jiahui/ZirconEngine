use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::super::common::UiAssetStringSelectionData;

#[derive(Clone, Default)]
pub(crate) struct UiAssetThemeSourceData {
    pub collection: UiAssetStringSelectionData,
    pub selected_source_reference: SharedString,
    pub selected_source_kind: SharedString,
    pub selected_source_token_count: i32,
    pub selected_source_rule_count: i32,
    pub selected_source_available: bool,
    pub can_promote_local: bool,
    pub selected_source_token_items: ModelRc<SharedString>,
    pub selected_source_rule_items: ModelRc<SharedString>,
    pub cascade_layer_items: ModelRc<SharedString>,
    pub cascade_token_items: ModelRc<SharedString>,
    pub cascade_rule_items: ModelRc<SharedString>,
    pub compare_items: ModelRc<SharedString>,
    pub merge_preview_items: ModelRc<SharedString>,
    pub rule_helper_items: ModelRc<SharedString>,
    pub refactor_items: ModelRc<SharedString>,
    pub promote_asset_id: SharedString,
    pub promote_document_id: SharedString,
    pub promote_display_name: SharedString,
    pub can_edit_promote_draft: bool,
    pub can_prune_duplicate_local_overrides: bool,
}
