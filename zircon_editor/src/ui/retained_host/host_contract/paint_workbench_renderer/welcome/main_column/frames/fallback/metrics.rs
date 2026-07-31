use super::super::super::super::super::super::data::FrameRect;
use super::super::super::super::super::super::paint_theme::current_host_metrics;
use super::super::super::super::layout::WELCOME_CONTENT_MAX_WIDTH;

pub(in super::super) struct WelcomeMainColumnFrameMetrics {
    pub(in super::super) content_x: f32,
    pub(in super::super) content_width: f32,
    pub(in super::super) top_inset: f32,
    pub(in super::super) section_gap: f32,
    pub(in super::super) form_section_gap: f32,
    pub(in super::super) hero_height: f32,
    pub(in super::super) status_height: f32,
    pub(in super::super) header_height: f32,
}

pub(in super::super) fn welcome_main_column_frame_metrics(
    main_panel: &FrameRect,
) -> WelcomeMainColumnFrameMetrics {
    let metrics = current_host_metrics();
    let content_inset = metrics.gap_l + metrics.gap_m * 2.0;
    WelcomeMainColumnFrameMetrics {
        content_x: main_panel.x + content_inset,
        content_width: (main_panel.width - content_inset * 2.0)
            .max(0.0)
            .min(WELCOME_CONTENT_MAX_WIDTH),
        top_inset: content_inset,
        section_gap: metrics.gap_l,
        form_section_gap: metrics.gap_l + metrics.gap_m + metrics.border_width * 2.0,
        hero_height: metrics.row_height * 3.0,
        status_height: metrics.row_height + metrics.border_width * 2.0,
        header_height: metrics.row_height + metrics.gap_m,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn welcome_main_column_fallback_metrics_derive_from_shared_spacing_and_rows() {
        let panel = FrameRect {
            x: 100.0,
            y: 20.0,
            width: 736.0,
            height: 520.0,
        };

        let welcome = welcome_main_column_frame_metrics(&panel);

        let metrics = current_host_metrics();
        let expected_inset = metrics.gap_l + metrics.gap_m * 2.0;
        assert_eq!(welcome.content_x, panel.x + expected_inset);
        assert_eq!(welcome.content_width, WELCOME_CONTENT_MAX_WIDTH);
        assert_eq!(welcome.top_inset, expected_inset);
        assert_eq!(welcome.section_gap, metrics.gap_l);
        assert_eq!(welcome.hero_height, metrics.row_height * 3.0);
        assert_eq!(
            welcome.status_height,
            metrics.row_height + metrics.border_width * 2.0
        );
        assert_eq!(welcome.header_height, metrics.row_height + metrics.gap_m);
    }
}
