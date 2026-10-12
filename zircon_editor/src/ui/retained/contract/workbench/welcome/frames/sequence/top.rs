use super::super::super::super::super::super::data::{FrameRect, WelcomePaneLayoutData};

use super::super::fallback::{
    header_fallback, hero_fallback, status_fallback, WelcomeMainColumnFrameMetrics,
};
use super::super::resolve::resolve_welcome_frame;

pub(super) struct WelcomeTopFrames {
    pub(super) hero: FrameRect,
    pub(super) status: FrameRect,
    pub(super) header: FrameRect,
}

pub(super) fn resolve_top_frames(
    layout: &WelcomePaneLayoutData,
    body: &FrameRect,
    main_panel: &FrameRect,
    metrics: &WelcomeMainColumnFrameMetrics,
) -> WelcomeTopFrames {
    let hero = resolve_welcome_frame(
        layout.hero_panel.as_ref(),
        layout.has_nodes,
        body,
        hero_fallback(metrics, main_panel),
        metrics,
    );
    let status = resolve_welcome_frame(
        layout.status_panel.as_ref(),
        layout.has_nodes,
        body,
        status_fallback(metrics, &hero),
        metrics,
    );
    let header = resolve_welcome_frame(
        layout.new_project_header_panel.as_ref(),
        layout.has_nodes,
        body,
        header_fallback(metrics, &status),
        metrics,
    );
    WelcomeTopFrames {
        hero,
        status,
        header,
    }
}
