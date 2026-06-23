use super::welcome_recent_pointer_route::WelcomeRecentPointerRoute;
use super::welcome_recent_pointer_route_intent::WelcomeRecentPointerRouteIntent;

pub(in crate::ui::retained_host::welcome_recent_pointer) fn to_public_route(
    target: WelcomeRecentPointerRouteIntent,
) -> WelcomeRecentPointerRoute {
    match target {
        WelcomeRecentPointerRouteIntent::Action {
            item_index,
            action,
            path,
        } => WelcomeRecentPointerRoute::Action {
            item_index,
            action,
            path,
        },
        WelcomeRecentPointerRouteIntent::Item(_) | WelcomeRecentPointerRouteIntent::ListSurface => {
            WelcomeRecentPointerRoute::ListSurface
        }
    }
}
