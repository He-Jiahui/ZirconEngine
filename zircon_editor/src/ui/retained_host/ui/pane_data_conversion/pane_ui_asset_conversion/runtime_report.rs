use crate::ui::asset_editor;
use crate::ui::retained_host as host_contract;

use super::string_selection::{
    to_host_contract_shared_string_list, to_host_contract_ui_asset_string_selection,
};

pub(super) fn to_host_contract_ui_asset_runtime_report(
    data: &mut asset_editor::UiAssetEditorPanePresentation,
) -> host_contract::UiAssetRuntimeReportData {
    host_contract::UiAssetRuntimeReportData {
        action_policy_items: to_host_contract_shared_string_list(std::mem::take(
            &mut data.action_policy_items,
        )),
        capability_explanation_items: to_host_contract_shared_string_list(std::mem::take(
            &mut data.capability_explanation_items,
        )),
        host_enforcement_items: to_host_contract_shared_string_list(std::mem::take(
            &mut data.host_enforcement_items,
        )),
        unsafe_action_guidance_items: to_host_contract_shared_string_list(std::mem::take(
            &mut data.unsafe_action_guidance_items,
        )),
        locale_preview: to_host_contract_ui_asset_string_selection(
            std::mem::take(&mut data.locale_preview_items),
            data.locale_preview_selected_index,
        ),
        locale_preview_selected_locale: std::mem::take(&mut data.locale_preview_selected_locale)
            .into(),
        locale_dependency_items: to_host_contract_shared_string_list(std::mem::take(
            &mut data.locale_dependency_items,
        )),
        locale_extraction_items: to_host_contract_shared_string_list(std::mem::take(
            &mut data.locale_extraction_items,
        )),
        locale_diagnostic_items: to_host_contract_shared_string_list(std::mem::take(
            &mut data.locale_diagnostic_items,
        )),
        resource_dependency_items: to_host_contract_shared_string_list(std::mem::take(
            &mut data.resource_dependency_items,
        )),
        resource_diagnostic_items: to_host_contract_shared_string_list(std::mem::take(
            &mut data.resource_diagnostic_items,
        )),
    }
}
