use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::metrics::WelcomeMainColumnFrameMetrics;
use super::frame::fallback;

pub(in super::super::super) fn hero_fallback(
    metrics: &WelcomeMainColumnFrameMetrics,
    main_panel: &FrameRect,
) -> FrameRect {
    fallback(
        metrics.content_x,
        main_panel.y + metrics.top_inset,
        metrics.content_width,
        metrics.hero_height,
    )
}

pub(in super::super::super) fn status_fallback(
    metrics: &WelcomeMainColumnFrameMetrics,
    hero: &FrameRect,
) -> FrameRect {
    fallback(
        metrics.content_x,
        hero.y + hero.height + metrics.section_gap,
        metrics.content_width,
        metrics.status_height,
    )
}

pub(in super::super::super) fn header_fallback(
    metrics: &WelcomeMainColumnFrameMetrics,
    status: &FrameRect,
) -> FrameRect {
    fallback(
        metrics.content_x,
        status.y + status.height + metrics.form_section_gap,
        metrics.content_width,
        metrics.header_height,
    )
}
