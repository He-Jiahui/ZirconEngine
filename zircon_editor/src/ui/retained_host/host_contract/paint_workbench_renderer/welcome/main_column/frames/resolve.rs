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
mod tests {
    use super::super::fallback::welcome_main_column_frame_metrics;
    use super::*;

    #[test]
    fn authoritative_asset_layout_does_not_resurrect_a_collapsed_frame() {
        let body = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 640.0,
            height: 360.0,
        };
        let main_panel = FrameRect {
            x: 220.0,
            y: 0.0,
            width: 420.0,
            height: 360.0,
        };
        let fallback = FrameRect {
            x: 240.0,
            y: 32.0,
            width: 360.0,
            height: 84.0,
        };
        let metrics = welcome_main_column_frame_metrics(&main_panel);

        let collapsed = resolve_welcome_frame(None, true, &body, fallback.clone(), &metrics);
        assert_eq!(collapsed.height, 0.0);

        let legacy = resolve_welcome_frame(None, false, &body, fallback, &metrics);
        assert_eq!(legacy.height, 84.0);
    }
}
