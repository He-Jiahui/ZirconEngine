use zircon_runtime_interface::ui::layout::UiFrame;

use crate::ui::retained_host::welcome_recent_geometry::{
    welcome_recent_content_height_with_metrics, WelcomeRecentLayoutMetrics,
};

use super::welcome_recent_pointer_layout::WelcomeRecentPointerLayout;

pub(in crate::ui::retained_host::welcome_recent_pointer) fn viewport_frame(
    layout: &WelcomeRecentPointerLayout,
) -> UiFrame {
    layout.viewport
}

pub(in crate::ui::retained_host::welcome_recent_pointer) fn content_height(
    item_count: usize,
    metrics: WelcomeRecentLayoutMetrics,
) -> f32 {
    welcome_recent_content_height_with_metrics(item_count, metrics)
}
