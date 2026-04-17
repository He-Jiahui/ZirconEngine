#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HierarchyPointerLayout {
    pub pane_width: f32,
    pub pane_height: f32,
    pub node_ids: Vec<String>,
}

impl Default for HierarchyPointerLayout {
    fn default() -> Self {
        Self {
            pane_width: 0.0,
            pane_height: 0.0,
            node_ids: Vec::new(),
        }
    }
}
