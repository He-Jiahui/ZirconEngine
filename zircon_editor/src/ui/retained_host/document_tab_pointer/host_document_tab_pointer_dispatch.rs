use super::host_document_tab_pointer_route::HostDocumentTabPointerRoute;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostDocumentTabPointerDispatch {
    pub route: Option<HostDocumentTabPointerRoute>,
}
