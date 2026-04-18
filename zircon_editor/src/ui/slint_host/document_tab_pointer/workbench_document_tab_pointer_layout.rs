use super::workbench_document_tab_pointer_surface::WorkbenchDocumentTabPointerSurface;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct WorkbenchDocumentTabPointerLayout {
    pub surfaces: Vec<WorkbenchDocumentTabPointerSurface>,
}
