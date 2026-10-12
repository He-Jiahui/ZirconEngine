use super::welcome_recent_pointer_route::WelcomeRecentPointerRoute;
use super::welcome_recent_pointer_state::WelcomeRecentPointerState;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WelcomeRecentPointerDispatch {
    pub route: Option<WelcomeRecentPointerRoute>,
    pub state: WelcomeRecentPointerState,
    pub changed: bool,
}
