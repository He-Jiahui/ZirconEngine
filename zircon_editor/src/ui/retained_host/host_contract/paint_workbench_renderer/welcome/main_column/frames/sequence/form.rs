use super::super::super::super::super::super::data::{FrameRect, PaneData};

use super::super::fallback::{
    actions_fallback, location_fallback, name_fallback, preview_fallback, validation_fallback,
    WelcomeMainColumnFrameMetrics,
};
use super::super::resolve::resolve_welcome_frame;

pub(super) struct WelcomeFormFrames {
    pub(super) name: FrameRect,
    pub(super) location: FrameRect,
    pub(super) preview: FrameRect,
    pub(super) validation: FrameRect,
    pub(super) actions: FrameRect,
}

pub(super) fn resolve_form_frames(
    pane: &PaneData,
    body: &FrameRect,
    metrics: &WelcomeMainColumnFrameMetrics,
    header: &FrameRect,
) -> WelcomeFormFrames {
    let name = resolve_welcome_frame(
        pane,
        body,
        "WelcomeProjectNameField",
        name_fallback(metrics, header),
        metrics,
    );
    let location = resolve_welcome_frame(
        pane,
        body,
        "WelcomeLocationField",
        location_fallback(metrics, &name),
        metrics,
    );
    let preview = resolve_welcome_frame(
        pane,
        body,
        "WelcomePreviewPanel",
        preview_fallback(metrics, &location),
        metrics,
    );
    let validation = resolve_welcome_frame(
        pane,
        body,
        "WelcomeValidationPanel",
        validation_fallback(metrics, &preview),
        metrics,
    );
    let actions = resolve_welcome_frame(
        pane,
        body,
        "WelcomeActionsRow",
        actions_fallback(metrics, &validation),
        metrics,
    );
    WelcomeFormFrames {
        name,
        location,
        preview,
        validation,
        actions,
    }
}
