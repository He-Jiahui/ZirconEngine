use super::welcome_recent_pointer_action::WelcomeRecentPointerAction;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::slint_host::welcome_recent_pointer) enum WelcomeRecentPointerTarget {
    Item(usize),
    Action {
        item_index: usize,
        action: WelcomeRecentPointerAction,
        path: String,
    },
    ListSurface,
}
