#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum HierarchyPointerRoute {
    Node { item_index: usize, node_id: String },
    ListSurface,
}
