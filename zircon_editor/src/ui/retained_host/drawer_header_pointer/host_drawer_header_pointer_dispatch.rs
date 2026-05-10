use super::host_drawer_header_pointer_route::HostDrawerHeaderPointerRoute;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostDrawerHeaderPointerDispatch {
    pub route: Option<HostDrawerHeaderPointerRoute>,
}
