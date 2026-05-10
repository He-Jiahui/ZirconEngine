use super::welcome_recent_pointer_action::WelcomeRecentPointerAction;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WelcomeRecentPointerRoute {
    Action {
        item_index: usize,
        action: WelcomeRecentPointerAction,
        path: String,
    },
    ListSurface,
}
