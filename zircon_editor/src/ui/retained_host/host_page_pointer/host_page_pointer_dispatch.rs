use super::host_page_pointer_route::HostPagePointerRoute;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostPagePointerDispatch {
    pub route: Option<HostPagePointerRoute>,
}
