use super::host_activity_rail_pointer_side::HostActivityRailPointerSide;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum HostActivityRailPointerTarget {
    Button {
        side: HostActivityRailPointerSide,
        item_index: usize,
        slot: String,
        instance_id: String,
    },
    Strip(HostActivityRailPointerSide),
}
