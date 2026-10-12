#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HostPagePointerRoute {
    Activate { item_index: usize },
    Close { item_index: usize },
}
