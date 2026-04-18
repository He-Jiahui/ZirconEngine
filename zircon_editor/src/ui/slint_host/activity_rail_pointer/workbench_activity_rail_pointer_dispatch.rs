use super::workbench_activity_rail_pointer_route::WorkbenchActivityRailPointerRoute;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorkbenchActivityRailPointerDispatch {
    pub route: Option<WorkbenchActivityRailPointerRoute>,
}
