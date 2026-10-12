use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::metrics::WelcomeMainColumnFrameMetrics;
use super::frame::fallback;

pub(in super::super::super) fn name_fallback(
    metrics: &WelcomeMainColumnFrameMetrics,
    header: &FrameRect,
) -> FrameRect {
    fallback(
        metrics.content_x,
        header.y + header.height + 16.0,
        metrics.content_width,
        56.0,
    )
}

pub(in super::super::super) fn location_fallback(
    metrics: &WelcomeMainColumnFrameMetrics,
    name: &FrameRect,
) -> FrameRect {
    fallback(
        metrics.content_x,
        name.y + name.height + 12.0,
        metrics.content_width,
        56.0,
    )
}

pub(in super::super::super) fn preview_fallback(
    metrics: &WelcomeMainColumnFrameMetrics,
    location: &FrameRect,
) -> FrameRect {
    fallback(
        metrics.content_x,
        location.y + location.height + 14.0,
        metrics.content_width,
        72.0,
    )
}

pub(in super::super::super) fn validation_fallback(
    metrics: &WelcomeMainColumnFrameMetrics,
    preview: &FrameRect,
) -> FrameRect {
    fallback(
        metrics.content_x,
        preview.y + preview.height + 10.0,
        metrics.content_width,
        36.0,
    )
}
