use super::workbench_host_page_pointer_route::WorkbenchHostPagePointerRoute;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorkbenchHostPagePointerDispatch {
    pub route: Option<WorkbenchHostPagePointerRoute>,
}
