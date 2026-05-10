use super::host_menu_pointer_route::HostMenuPointerRoute;
use super::host_menu_pointer_state::HostMenuPointerState;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostMenuPointerDispatch {
    pub route: Option<HostMenuPointerRoute>,
    pub state: HostMenuPointerState,
    pub action_id: Option<String>,
}
