use super::workbench_document_tab_pointer_route::WorkbenchDocumentTabPointerRoute;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorkbenchDocumentTabPointerDispatch {
    pub route: Option<WorkbenchDocumentTabPointerRoute>,
}
