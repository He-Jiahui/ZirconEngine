#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryPageDependency {
    pub page_id: u32,
    pub parent_page_id: Option<u32>,
    /// Stable child list from cooked VG data; runtime may derive its parent map from either side.
    pub child_page_ids: Vec<u32>,
}
