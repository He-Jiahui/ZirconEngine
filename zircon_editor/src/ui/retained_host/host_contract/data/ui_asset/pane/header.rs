use crate::ui::retained_host::primitives::SharedString;

#[derive(Clone, Default)]
pub(crate) struct UiAssetPaneHeaderData {
    pub asset_id: SharedString,
    pub mode: SharedString,
    pub status: SharedString,
    pub selection: SharedString,
    pub shell_state: SharedString,
    pub emergency_summary: SharedString,
}
