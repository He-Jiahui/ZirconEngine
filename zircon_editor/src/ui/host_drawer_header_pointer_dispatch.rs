use super::host_drawer_header_pointer_route::HostDrawerHeaderPointerRoute;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct HostDrawerHeaderPointerDispatch {
    pub route: Option<HostDrawerHeaderPointerRoute>,
}
