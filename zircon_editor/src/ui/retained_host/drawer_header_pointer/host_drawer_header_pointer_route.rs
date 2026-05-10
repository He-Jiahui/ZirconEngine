#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum HostDrawerHeaderPointerRoute {
    Tab {
        surface_key: String,
        item_index: usize,
        slot: String,
        instance_id: String,
    },
}
