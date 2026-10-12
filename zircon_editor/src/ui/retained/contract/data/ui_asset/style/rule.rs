use crate::ui::retained_host::primitives::{ModelRc, SharedString};

#[derive(Clone, Default)]
pub(crate) struct UiAssetStyleRuleData {
    pub items: ModelRc<SharedString>,
    pub selected_index: i32,
    pub selected_selector: SharedString,
    pub can_edit: bool,
    pub can_delete: bool,
}
