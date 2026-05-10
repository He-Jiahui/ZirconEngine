#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum HostDrawerHeaderPointerTarget {
    Tab {
        surface_key: String,
        item_index: usize,
        slot: String,
        instance_id: String,
    },
}
