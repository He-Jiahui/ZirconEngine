use super::super::super::super::super::super::data::FrameRect;
use super::super::super::super::layout::WELCOME_CONTENT_MAX_WIDTH;

pub(in super::super) struct WelcomeMainColumnFrameMetrics {
    pub(in super::super) content_x: f32,
    pub(in super::super) content_width: f32,
}

pub(in super::super) fn welcome_main_column_frame_metrics(
    main_panel: &FrameRect,
) -> WelcomeMainColumnFrameMetrics {
    WelcomeMainColumnFrameMetrics {
        content_x: main_panel.x + 28.0,
        content_width: (main_panel.width - 56.0)
            .max(0.0)
            .min(WELCOME_CONTENT_MAX_WIDTH),
    }
}
