use super::super::super::super::super::super::data::{FrameRect, WelcomePaneLayoutData};

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
    layout: &WelcomePaneLayoutData,
    body: &FrameRect,
    metrics: &WelcomeMainColumnFrameMetrics,
    header: &FrameRect,
) -> WelcomeFormFrames {
    let name = resolve_welcome_frame(
        layout.project_name_field.as_ref(),
        layout.has_nodes,
        body,
        name_fallback(metrics, header),
        metrics,
    );
    let location = resolve_welcome_frame(
        layout.location_field.as_ref(),
        layout.has_nodes,
        body,
        location_fallback(metrics, &name),
        metrics,
    );
    let preview = resolve_welcome_frame(
        layout.preview_panel.as_ref(),
        layout.has_nodes,
        body,
        preview_fallback(metrics, &location),
        metrics,
    );
    let validation = resolve_welcome_frame(
        layout.validation_panel.as_ref(),
        layout.has_nodes,
        body,
        validation_fallback(metrics, &preview),
        metrics,
    );
    WelcomeFormFrames {
        preview,
        validation,
    }
}
