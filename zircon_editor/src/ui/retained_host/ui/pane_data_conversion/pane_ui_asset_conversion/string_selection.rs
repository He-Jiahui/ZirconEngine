use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::{ModelRc, SharedString};

pub(super) fn to_host_contract_shared_string_list(items: Vec<String>) -> ModelRc<SharedString> {
    model_rc(items.into_iter().map(SharedString::from).collect())
}

pub(super) fn to_host_contract_ui_asset_string_selection(
    items: Vec<String>,
    selected_index: i32,
) -> host_contract::UiAssetStringSelectionData {
    host_contract::UiAssetStringSelectionData {
        items: to_host_contract_shared_string_list(items),
        selected_index,
    }
}
