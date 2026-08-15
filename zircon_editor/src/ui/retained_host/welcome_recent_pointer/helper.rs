use zircon_runtime_interface::ui::{
    event_ui::{UiRouteId, UiStateFlags},
    layout::UiFrame,
};

use crate::ui::retained_host::welcome_recent_geometry::{
    welcome_recent_content_height_with_metrics, welcome_recent_viewport_with_metrics,
    WelcomeRecentLayoutMetrics,
};

use super::constants::{VIEWPORT_NODE_ID, WELCOME_RECENT_ROUTE_ID_BASE};
use super::welcome_recent_pointer_layout::WelcomeRecentPointerLayout;

pub(in crate::ui::retained_host::welcome_recent_pointer) fn viewport_frame(
    layout: &WelcomeRecentPointerLayout,
    metrics: WelcomeRecentLayoutMetrics,
) -> UiFrame {
    welcome_recent_viewport_with_metrics(layout.pane_size, metrics)
}

pub(in crate::ui::retained_host::welcome_recent_pointer) fn content_height(
    item_count: usize,
    metrics: WelcomeRecentLayoutMetrics,
) -> f32 {
    welcome_recent_content_height_with_metrics(item_count, metrics)
}

pub(in crate::ui::retained_host::welcome_recent_pointer) fn list_surface_route_id() -> UiRouteId {
    UiRouteId::new(WELCOME_RECENT_ROUTE_ID_BASE + VIEWPORT_NODE_ID.0)
}

pub(in crate::ui::retained_host::welcome_recent_pointer) fn base_state(
    interactive: bool,
) -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: interactive,
        clickable: interactive,
        hoverable: interactive,
        focusable: false,
        pressed: false,
        checked: false,
        dirty: false,
    }
}
