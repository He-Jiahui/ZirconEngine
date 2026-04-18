use super::workbench_activity_rail_pointer_side::WorkbenchActivityRailPointerSide;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WorkbenchActivityRailPointerRoute {
    Button {
        side: WorkbenchActivityRailPointerSide,
        item_index: usize,
        slot: String,
        instance_id: String,
    },
    Strip(WorkbenchActivityRailPointerSide),
}
