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
#[path = "tests/metrics.rs"]
mod tests;
