#[derive(Clone, PartialEq)]
pub(crate) struct HostPageOverflowMenuStateData {
    pub open: bool,
    pub hovered_page_index: i32,
    /// Vertical content displacement inside the bounded overflow popup viewport.
    pub scroll_offset: f32,
}

impl Default for HostPageOverflowMenuStateData {
    fn default() -> Self {
        Self {
            open: false,
            hovered_page_index: -1,
            scroll_offset: 0.0,
        }
    }
}
