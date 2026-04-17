use super::workbench_drawer_header_pointer_surface::WorkbenchDrawerHeaderPointerSurface;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct WorkbenchDrawerHeaderPointerLayout {
    pub surfaces: Vec<WorkbenchDrawerHeaderPointerSurface>,
}
