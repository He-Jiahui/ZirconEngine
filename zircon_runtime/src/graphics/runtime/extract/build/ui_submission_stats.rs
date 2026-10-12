use zircon_runtime_interface::ui::surface::UiRenderCommandKind;

use super::super::super::frame_submission_context::UiSubmissionStats;
use crate::core::framework::render::UiRenderSubmission;

pub(super) fn compute_ui_submission_stats(submission: &UiRenderSubmission) -> UiSubmissionStats {
    let mut stats = UiSubmissionStats::default();
    for command in submission.commands() {
        stats.record_command();
        if matches!(command.kind, UiRenderCommandKind::Quad) {
            stats.record_quad();
        }
        if command.text.is_some() {
            stats.record_text_payload();
        }
        if command.image.is_some() {
            stats.record_image_payload();
        }
        if command.clip_frame.is_some() {
            stats.record_clipped_command();
        }
    }
    stats
}
