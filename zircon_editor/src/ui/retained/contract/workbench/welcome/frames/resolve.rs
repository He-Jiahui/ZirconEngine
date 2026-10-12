use super::super::super::super::super::data::FrameRect;
use super::super::super::layout::{constrain_welcome_content, translated_welcome_frame};

use super::fallback::WelcomeMainColumnFrameMetrics;

pub(super) fn resolve_welcome_frame(
    source: Option<&FrameRect>,
    asset_layout_is_authoritative: bool,
    body: &FrameRect,
    fallback: FrameRect,
    metrics: &WelcomeMainColumnFrameMetrics,
) -> FrameRect {
    let resolved = match translated_welcome_frame(source, body) {
        Some(frame) => frame,
        None if asset_layout_is_authoritative => FrameRect::default(),
        None => fallback,
    };
    constrain_welcome_content(resolved, metrics.content_x, metrics.content_width)
}

#[cfg(test)]
#[path = "tests/resolve.rs"]
mod tests;
