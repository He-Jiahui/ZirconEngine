use crate::ui::asset_editor;
use crate::ui::retained_host as host_contract;

use super::string_selection::{
    to_host_contract_shared_string_list, to_host_contract_ui_asset_string_selection,
};

pub(super) fn to_host_contract_ui_asset_style_panel(
    data: &mut asset_editor::UiAssetEditorPanePresentation,
) -> host_contract::UiAssetStylePanelData {
    host_contract::UiAssetStylePanelData {
        states: host_contract::UiAssetStyleStateData {
            hover: data.style_state_hover,
            focus: data.style_state_focus,
            pressed: data.style_state_pressed,
            disabled: data.style_state_disabled,
            selected: data.style_state_selected,
        },
        class_items: to_host_contract_shared_string_list(std::mem::take(
            &mut data.style_class_items,
        )),
        theme_source: host_contract::UiAssetThemeSourceData {
            collection: to_host_contract_ui_asset_string_selection(
                std::mem::take(&mut data.theme_source_items),
                data.theme_source_selected_index,
            ),
            selected_source_reference: std::mem::take(&mut data.theme_selected_source_reference)
                .into(),
            selected_source_kind: std::mem::take(&mut data.theme_selected_source_kind).into(),
            selected_source_token_count: data.theme_selected_source_token_count,
            selected_source_rule_count: data.theme_selected_source_rule_count,
            selected_source_available: data.theme_selected_source_available,
            can_promote_local: data.theme_can_promote_local,
            selected_source_token_items: to_host_contract_shared_string_list(std::mem::take(
                &mut data.theme_selected_source_token_items,
            )),
            selected_source_rule_items: to_host_contract_shared_string_list(std::mem::take(
                &mut data.theme_selected_source_rule_items,
            )),
            cascade_layer_items: to_host_contract_shared_string_list(std::mem::take(
                &mut data.theme_cascade_layer_items,
            )),
            cascade_token_items: to_host_contract_shared_string_list(std::mem::take(
                &mut data.theme_cascade_token_items,
            )),
            cascade_rule_items: to_host_contract_shared_string_list(std::mem::take(
                &mut data.theme_cascade_rule_items,
            )),
            compare_items: to_host_contract_shared_string_list(std::mem::take(
                &mut data.theme_compare_items,
            )),
            merge_preview_items: to_host_contract_shared_string_list(std::mem::take(
                &mut data.theme_merge_preview_items,
            )),
            rule_helper_items: to_host_contract_shared_string_list(std::mem::take(
                &mut data.theme_rule_helper_items,
            )),
            refactor_items: to_host_contract_shared_string_list(std::mem::take(
                &mut data.theme_refactor_items,
            )),
            promote_asset_id: std::mem::take(&mut data.theme_promote_asset_id).into(),
            promote_document_id: std::mem::take(&mut data.theme_promote_document_id).into(),
            promote_display_name: std::mem::take(&mut data.theme_promote_display_name).into(),
            can_edit_promote_draft: data.theme_can_edit_promote_draft,
            can_prune_duplicate_local_overrides: data.theme_can_prune_duplicate_local_overrides,
        },
        rule: host_contract::UiAssetStyleRuleData {
            items: to_host_contract_shared_string_list(std::mem::take(&mut data.style_rule_items)),
            selected_index: data.style_rule_selected_index,
            selected_selector: std::mem::take(&mut data.style_selected_rule_selector).into(),
            can_edit: data.style_can_edit_rule,
            can_delete: data.style_can_delete_rule,
        },
        matched_rule: host_contract::UiAssetMatchedStyleRuleData {
            collection: to_host_contract_ui_asset_string_selection(
                std::mem::take(&mut data.style_matched_rule_items),
                data.style_matched_rule_selected_index,
            ),
            selected_origin: std::mem::take(&mut data.style_selected_matched_rule_origin).into(),
            selected_selector: std::mem::take(&mut data.style_selected_matched_rule_selector)
                .into(),
            selected_specificity: data.style_selected_matched_rule_specificity,
            selected_source_order: data.style_selected_matched_rule_source_order,
            selected_declaration_items: to_host_contract_shared_string_list(std::mem::take(
                &mut data.style_selected_matched_rule_declaration_items,
            )),
        },
        rule_declaration: host_contract::UiAssetStyleRuleDeclarationData {
            items: to_host_contract_shared_string_list(std::mem::take(
                &mut data.style_rule_declaration_items,
            )),
            selected_index: data.style_rule_declaration_selected_index,
            selected_path: std::mem::take(&mut data.style_selected_rule_declaration_path).into(),
            selected_value: std::mem::take(&mut data.style_selected_rule_declaration_value).into(),
            can_edit: data.style_can_edit_rule_declaration,
            can_delete: data.style_can_delete_rule_declaration,
        },
        token: host_contract::UiAssetStyleTokenData {
            items: to_host_contract_shared_string_list(std::mem::take(&mut data.style_token_items)),
            selected_index: data.style_token_selected_index,
            selected_name: std::mem::take(&mut data.style_selected_token_name).into(),
            selected_value: std::mem::take(&mut data.style_selected_token_value).into(),
            can_edit: data.style_can_edit_token,
            can_delete: data.style_can_delete_token,
        },
        can_create_rule: data.can_create_rule,
        can_extract_rule: data.can_extract_rule,
        stylesheet_items: to_host_contract_shared_string_list(std::mem::take(
            &mut data.stylesheet_items,
        )),
    }
}
