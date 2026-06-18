use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::common::UiAssetStringSelectionData;

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

#[derive(Clone, Default)]
pub(crate) struct UiAssetStyleRuleData {
    pub items: ModelRc<SharedString>,
    pub selected_index: i32,
    pub selected_selector: SharedString,
    pub can_edit: bool,
    pub can_delete: bool,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetMatchedStyleRuleData {
    pub collection: UiAssetStringSelectionData,
    pub selected_origin: SharedString,
    pub selected_selector: SharedString,
    pub selected_specificity: i32,
    pub selected_source_order: i32,
    pub selected_declaration_items: ModelRc<SharedString>,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetStyleRuleDeclarationData {
    pub items: ModelRc<SharedString>,
    pub selected_index: i32,
    pub selected_path: SharedString,
    pub selected_value: SharedString,
    pub can_edit: bool,
    pub can_delete: bool,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetStyleTokenData {
    pub items: ModelRc<SharedString>,
    pub selected_index: i32,
    pub selected_name: SharedString,
    pub selected_value: SharedString,
    pub can_edit: bool,
    pub can_delete: bool,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetStyleStateData {
    pub hover: bool,
    pub focus: bool,
    pub pressed: bool,
    pub disabled: bool,
    pub selected: bool,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetStylePanelData {
    pub states: UiAssetStyleStateData,
    pub class_items: ModelRc<SharedString>,
    pub theme_source: UiAssetThemeSourceData,
    pub rule: UiAssetStyleRuleData,
    pub matched_rule: UiAssetMatchedStyleRuleData,
    pub rule_declaration: UiAssetStyleRuleDeclarationData,
    pub token: UiAssetStyleTokenData,
    pub can_create_rule: bool,
    pub can_extract_rule: bool,
    pub stylesheet_items: ModelRc<SharedString>,
}
