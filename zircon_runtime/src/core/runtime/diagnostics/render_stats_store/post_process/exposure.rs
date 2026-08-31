use crate::core::framework::render::RenderExposureReadbackReport;

use super::super::{record_bool, record_bytes, record_count, DiagnosticStore};

const EXPOSURE_READBACK_TAGS: &[&str] = &["render", "post_process", "exposure", "readback"];

pub(super) fn record(
    store: &mut DiagnosticStore,
    frame_index: u64,
    report: RenderExposureReadbackReport,
) {
    record_bool(
        store,
        "render.post_process.exposure.readback.available",
        frame_index,
        report.available,
        EXPOSURE_READBACK_TAGS,
    );
    record_bool(
        store,
        "render.post_process.exposure.readback.history_valid",
        frame_index,
        report.history_valid(),
        EXPOSURE_READBACK_TAGS,
    );
    record_bytes(
        store,
        "render.post_process.exposure.readback.byte_len",
        frame_index,
        report.byte_len as u64,
        EXPOSURE_READBACK_TAGS,
    );
    record_bytes(
        store,
        "render.post_process.exposure.readback.expected_byte_len",
        frame_index,
        report.expected_byte_len as u64,
        EXPOSURE_READBACK_TAGS,
    );
    record_bool(
        store,
        "render.post_process.exposure.readback.invalid_byte_len",
        frame_index,
        report.invalid_byte_len,
        EXPOSURE_READBACK_TAGS,
    );
    record_count(
        store,
        "render.post_process.exposure.readback.invalid_word_count",
        frame_index,
        report.invalid_word_count,
        EXPOSURE_READBACK_TAGS,
    );
    record_count(
        store,
        "render.post_process.exposure.readback.multiplier_micro",
        frame_index,
        report.multiplier_micro(),
        EXPOSURE_READBACK_TAGS,
    );
    record_count(
        store,
        "render.post_process.exposure.readback.valid_flag_micro",
        frame_index,
        report.valid_flag_micro(),
        EXPOSURE_READBACK_TAGS,
    );
}
