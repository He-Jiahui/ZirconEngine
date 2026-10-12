#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HostDrawerHeaderPointerRoute {
    Tab {
        surface_index: usize,
        item_index: usize,
    },
}
