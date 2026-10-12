#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HierarchyPointerRoute {
    Node { item_index: usize },
    ListSurface,
}
