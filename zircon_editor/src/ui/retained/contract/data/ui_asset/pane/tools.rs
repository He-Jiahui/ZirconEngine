use crate::ui::retained_host::primitives::SharedString;

#[derive(Clone, Default)]
pub(crate) struct UiAssetDesignerToolStateData {
    pub mode: SharedString,
    pub can_select: bool,
    pub can_resize_slot: bool,
    pub can_preview_interact: bool,
}
