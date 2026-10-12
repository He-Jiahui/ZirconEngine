use super::host_document_tab_pointer_route::HostDocumentTabPointerRoute;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct HostDocumentTabPointerDispatch {
    pub route: Option<HostDocumentTabPointerRoute>,
}
