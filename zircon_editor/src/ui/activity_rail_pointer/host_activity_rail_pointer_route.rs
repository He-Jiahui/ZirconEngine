use super::host_activity_rail_pointer_side::HostActivityRailPointerSide;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HostActivityRailPointerRoute {
    Button {
        side: HostActivityRailPointerSide,
        item_index: usize,
    },
    Strip(HostActivityRailPointerSide),
}
