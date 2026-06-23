use super::welcome_recent_pointer_action::WelcomeRecentPointerAction;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WelcomeRecentPointerRouteIntent {
    Item(usize),
    Action {
        item_index: usize,
        action: WelcomeRecentPointerAction,
        path: String,
    },
    ListSurface,
}
