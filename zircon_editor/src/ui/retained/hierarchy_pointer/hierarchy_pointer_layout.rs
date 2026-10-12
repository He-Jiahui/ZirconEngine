#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct HierarchyPointerLayout {
    pub pane_width: f32,
    pub pane_height: f32,
    pub item_count: usize,
}

impl Default for HierarchyPointerLayout {
    fn default() -> Self {
        Self {
            pane_width: 0.0,
            pane_height: 0.0,
            item_count: 0,
        }
    }
}
