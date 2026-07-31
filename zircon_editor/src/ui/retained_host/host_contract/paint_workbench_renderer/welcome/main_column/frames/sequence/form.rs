use super::super::super::super::super::super::data::{FrameRect, PaneData};

use super::super::fallback::{
    location_fallback, name_fallback, preview_fallback, validation_fallback,
    WelcomeMainColumnFrameMetrics,
};
use super::super::resolve::resolve_welcome_frame;

pub(super) struct WelcomeFormFrames {
    pub(super) preview: FrameRect,
    pub(super) validation: FrameRect,
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
    WelcomeFormFrames {
        preview,
        validation,
    }
}
