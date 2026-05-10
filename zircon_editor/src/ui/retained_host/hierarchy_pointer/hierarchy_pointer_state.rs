#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HierarchyPointerState {
    pub hovered_item_index: Option<usize>,
    pub scroll_offset: f32,
}
