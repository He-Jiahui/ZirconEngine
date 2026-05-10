use super::host_activity_rail_pointer_route::HostActivityRailPointerRoute;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostActivityRailPointerDispatch {
    pub route: Option<HostActivityRailPointerRoute>,
}
