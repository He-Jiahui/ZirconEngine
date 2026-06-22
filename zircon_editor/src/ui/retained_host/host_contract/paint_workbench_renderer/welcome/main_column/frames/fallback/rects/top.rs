use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::metrics::WelcomeMainColumnFrameMetrics;
use super::frame::fallback;

pub(in super::super::super) fn hero_fallback(
    metrics: &WelcomeMainColumnFrameMetrics,
    main_panel: &FrameRect,
) -> FrameRect {
    fallback(
        metrics.content_x,
        main_panel.y + 28.0,
        metrics.content_width,
        84.0,
    )
}

pub(in super::super::super) fn status_fallback(
    metrics: &WelcomeMainColumnFrameMetrics,
    hero: &FrameRect,
) -> FrameRect {
    fallback(
        metrics.content_x,
        hero.y + hero.height + 12.0,
        metrics.content_width,
        30.0,
    )
}

pub(in super::super::super) fn header_fallback(
    metrics: &WelcomeMainColumnFrameMetrics,
    status: &FrameRect,
) -> FrameRect {
    fallback(
        metrics.content_x,
        status.y + status.height + 22.0,
        metrics.content_width,
        46.0,
    )
}
