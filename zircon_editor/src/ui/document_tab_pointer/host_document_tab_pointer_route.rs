#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HostDocumentTabPointerRoute {
    ActivateTab {
        surface_index: usize,
        item_index: usize,
    },
    CloseTab {
        surface_index: usize,
        item_index: usize,
    },
}
