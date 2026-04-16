use super::welcome_recent_pointer_route::WelcomeRecentPointerRoute;
use super::welcome_recent_pointer_target::WelcomeRecentPointerTarget;

pub(in crate::host::slint_host::welcome_recent_pointer) fn to_public_route(
    target: WelcomeRecentPointerTarget,
) -> WelcomeRecentPointerRoute {
    match target {
        WelcomeRecentPointerTarget::Action {
            item_index,
            action,
            path,
        } => WelcomeRecentPointerRoute::Action {
            item_index,
            action,
            path,
        },
        WelcomeRecentPointerTarget::Item(_) | WelcomeRecentPointerTarget::ListSurface => {
            WelcomeRecentPointerRoute::ListSurface
        }
    }
}
