use super::host_document_tab_pointer_surface::HostDocumentTabPointerSurface;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HostDocumentTabPointerLayout {
    pub surfaces: Vec<HostDocumentTabPointerSurface>,
}
