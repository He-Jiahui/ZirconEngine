use super::workbench_menu_pointer_route::WorkbenchMenuPointerRoute;
use super::workbench_menu_pointer_state::WorkbenchMenuPointerState;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorkbenchMenuPointerDispatch {
    pub route: Option<WorkbenchMenuPointerRoute>,
    pub state: WorkbenchMenuPointerState,
    pub action_id: Option<String>,
}
