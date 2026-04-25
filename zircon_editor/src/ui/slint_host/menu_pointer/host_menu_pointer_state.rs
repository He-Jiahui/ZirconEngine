#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HostMenuPointerState {
    pub open_menu_index: Option<usize>,
    pub hovered_menu_index: Option<usize>,
    pub hovered_item_index: Option<usize>,
    pub popup_scroll_offset: f32,
}
