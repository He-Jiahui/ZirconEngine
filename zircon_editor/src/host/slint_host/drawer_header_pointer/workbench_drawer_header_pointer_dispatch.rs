use super::workbench_drawer_header_pointer_route::WorkbenchDrawerHeaderPointerRoute;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorkbenchDrawerHeaderPointerDispatch {
    pub route: Option<WorkbenchDrawerHeaderPointerRoute>,
}
