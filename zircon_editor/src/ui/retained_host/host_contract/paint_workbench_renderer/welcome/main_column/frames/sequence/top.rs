use super::super::super::super::super::super::data::{FrameRect, PaneData};

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
    pane: &PaneData,
    body: &FrameRect,
    main_panel: &FrameRect,
    metrics: &WelcomeMainColumnFrameMetrics,
) -> WelcomeTopFrames {
    let hero = resolve_welcome_frame(
        pane,
        body,
        "WelcomeHeroPanel",
        hero_fallback(metrics, main_panel),
        metrics,
    );
    let status = resolve_welcome_frame(
        pane,
        body,
        "WelcomeStatusPanel",
        status_fallback(metrics, &hero),
        metrics,
    );
    let header = resolve_welcome_frame(
        pane,
        body,
        "WelcomeNewProjectHeaderPanel",
        header_fallback(metrics, &status),
        metrics,
    );
    WelcomeTopFrames {
        hero,
        status,
        header,
    }
}
