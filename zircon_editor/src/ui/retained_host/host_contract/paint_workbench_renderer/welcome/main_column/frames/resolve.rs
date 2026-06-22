use super::super::super::super::super::data::{FrameRect, PaneData};
use super::super::super::layout::{constrain_welcome_content, welcome_node_frame};

use super::fallback::WelcomeMainColumnFrameMetrics;

pub(super) fn resolve_welcome_frame(
    pane: &PaneData,
    body: &FrameRect,
    control_id: &str,
    fallback: FrameRect,
    metrics: &WelcomeMainColumnFrameMetrics,
) -> FrameRect {
    constrain_welcome_content(
        welcome_node_frame(pane, body, control_id).unwrap_or(fallback),
        metrics.content_x,
        metrics.content_width,
    )
}
