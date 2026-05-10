#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum HierarchyPointerTarget {
    Node { item_index: usize, node_id: String },
    ListSurface,
}
