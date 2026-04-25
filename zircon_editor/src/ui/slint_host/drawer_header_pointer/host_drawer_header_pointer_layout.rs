use super::host_drawer_header_pointer_surface::HostDrawerHeaderPointerSurface;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HostDrawerHeaderPointerLayout {
    pub surfaces: Vec<HostDrawerHeaderPointerSurface>,
}
