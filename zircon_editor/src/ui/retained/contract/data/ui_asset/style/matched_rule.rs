use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::super::common::UiAssetStringSelectionData;

#[derive(Clone, Default)]
pub(crate) struct UiAssetMatchedStyleRuleData {
    pub collection: UiAssetStringSelectionData,
    pub selected_origin: SharedString,
    pub selected_selector: SharedString,
    pub selected_specificity: i32,
    pub selected_source_order: i32,
    pub selected_declaration_items: ModelRc<SharedString>,
}
