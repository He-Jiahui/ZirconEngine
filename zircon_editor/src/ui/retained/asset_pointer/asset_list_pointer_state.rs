#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct AssetListPointerState {
    pub hovered_row_index: Option<usize>,
    pub scroll_offset: f32,
}
