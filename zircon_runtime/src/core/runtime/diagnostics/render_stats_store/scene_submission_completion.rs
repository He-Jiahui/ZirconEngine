use crate::core::framework::render::{RenderSceneSubmissionCompletionStatus, RenderStats};

use super::{record_bool, record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    let report = stats.last_scene_submission_completion_report;
    let tags = &["render", "submission", "completion"];

    record_count(
        store,
        "render.submission.completion.status_code",
        frame_index,
        report.status.code() as usize,
        tags,
    );
    record_count(
        store,
        "render.submission.completion.failure_code",
        frame_index,
        report.failure.code() as usize,
        tags,
    );
    record_bool(
        store,
        "render.submission.completion.completed",
        frame_index,
        report.status == RenderSceneSubmissionCompletionStatus::Completed,
        tags,
    );
    for (path, value) in [
        (
            "render.submission.completion.frame_generation",
            report.frame_generation,
        ),
        (
            "render.submission.completion.submission_sequence",
            report.submission.map_or(0, |ticket| ticket.sequence()),
        ),
        (
            "render.submission.completion.poll_sequence",
            report
                .observed_after_poll
                .map_or(0, |receipt| receipt.sequence()),
        ),
        (
            "render.submission.completion.device_generation",
            report
                .submission
                .map_or(0, |ticket| ticket.generation().raw()),
        ),
    ] {
        record_count(
            store,
            path,
            frame_index,
            usize::try_from(value).unwrap_or(usize::MAX),
            tags,
        );
    }
    for (path, value) in [
        (
            "render.submission.completion.pending_submission_count",
            report.pending_submission_count,
        ),
        (
            "render.submission.completion.tracking_capacity",
            report.tracking_capacity,
        ),
        (
            "render.submission.completion.last_poll_observed_submission_count",
            report.last_poll_observed_submission_count,
        ),
        (
            "render.submission.completion.last_poll_terminal_submission_count",
            report.last_poll_terminal_submission_count,
        ),
    ] {
        record_count(store, path, frame_index, value, tags);
    }
}
