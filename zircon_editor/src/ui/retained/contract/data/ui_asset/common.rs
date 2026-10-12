use crate::ui::retained_host::primitives::{ModelRc, SharedString};

#[derive(Clone, Default)]
pub(crate) struct UiAssetStringSelectionData {
    pub items: ModelRc<SharedString>,
    pub selected_index: i32,
}
