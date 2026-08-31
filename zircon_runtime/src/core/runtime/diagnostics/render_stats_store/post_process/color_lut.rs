use crate::core::framework::render::RenderColorLutReadbackReport;

use super::super::{record_bool, record_bytes, record_count, DiagnosticStore};

const COLOR_LUT_READBACK_TAGS: &[&str] = &["render", "post_process", "color_lut", "readback"];
const COLOR_LUT_READBACK_IDENTITY_TAGS: &[&str] = &[
    "render",
    "post_process",
    "color_lut",
    "readback",
    "identity",
];
const COLOR_LUT_READBACK_REFERENCE_TAGS: &[&str] = &[
    "render",
    "post_process",
    "color_lut",
    "readback",
    "reference",
];

pub(super) fn record(
    store: &mut DiagnosticStore,
    frame_index: u64,
    report: RenderColorLutReadbackReport,
) {
    record_bool(
        store,
        "render.post_process.color_lut.readback.available",
        frame_index,
        report.available,
        COLOR_LUT_READBACK_TAGS,
    );
    record_bool(
        store,
        "render.post_process.color_lut.readback.identity_within_epsilon",
        frame_index,
        report.identity_within_epsilon(),
        COLOR_LUT_READBACK_IDENTITY_TAGS,
    );
    record_count(
        store,
        "render.post_process.color_lut.readback.reference_kind",
        frame_index,
        report.reference.diagnostic_id() as usize,
        COLOR_LUT_READBACK_REFERENCE_TAGS,
    );
    record_bool(
        store,
        "render.post_process.color_lut.readback.reference_within_epsilon",
        frame_index,
        report.reference_within_epsilon(),
        COLOR_LUT_READBACK_REFERENCE_TAGS,
    );
    record_count(
        store,
        "render.post_process.color_lut.readback.width",
        frame_index,
        report.size[0] as usize,
        COLOR_LUT_READBACK_TAGS,
    );
    record_count(
        store,
        "render.post_process.color_lut.readback.height",
        frame_index,
        report.size[1] as usize,
        COLOR_LUT_READBACK_TAGS,
    );
    record_count(
        store,
        "render.post_process.color_lut.readback.depth",
        frame_index,
        report.size[2] as usize,
        COLOR_LUT_READBACK_TAGS,
    );
    record_bytes(
        store,
        "render.post_process.color_lut.readback.byte_len",
        frame_index,
        report.byte_len as u64,
        COLOR_LUT_READBACK_TAGS,
    );
    record_bytes(
        store,
        "render.post_process.color_lut.readback.expected_byte_len",
        frame_index,
        report.expected_byte_len as u64,
        COLOR_LUT_READBACK_TAGS,
    );
    record_count(
        store,
        "render.post_process.color_lut.readback.sample_count",
        frame_index,
        report.sample_count,
        COLOR_LUT_READBACK_TAGS,
    );
    record_bool(
        store,
        "render.post_process.color_lut.readback.invalid_byte_len",
        frame_index,
        report.invalid_byte_len,
        COLOR_LUT_READBACK_TAGS,
    );
    record_count(
        store,
        "render.post_process.color_lut.readback.invalid_sample_count",
        frame_index,
        report.invalid_sample_count,
        COLOR_LUT_READBACK_TAGS,
    );
    record_count(
        store,
        "render.post_process.color_lut.readback.reference_max_abs_error_micro",
        frame_index,
        report.max_abs_error_micro as usize,
        COLOR_LUT_READBACK_REFERENCE_TAGS,
    );
    record_count(
        store,
        "render.post_process.color_lut.readback.reference_out_of_tolerance_sample_count",
        frame_index,
        report.out_of_tolerance_sample_count,
        COLOR_LUT_READBACK_REFERENCE_TAGS,
    );
    record_count(
        store,
        "render.post_process.color_lut.readback.identity_max_abs_error_micro",
        frame_index,
        report.identity_max_abs_error_micro as usize,
        COLOR_LUT_READBACK_IDENTITY_TAGS,
    );
    record_count(
        store,
        "render.post_process.color_lut.readback.identity_out_of_tolerance_sample_count",
        frame_index,
        report.identity_out_of_tolerance_sample_count,
        COLOR_LUT_READBACK_IDENTITY_TAGS,
    );
    record_count(
        store,
        "render.post_process.color_lut.readback.alpha_out_of_tolerance_sample_count",
        frame_index,
        report.alpha_out_of_tolerance_sample_count,
        COLOR_LUT_READBACK_IDENTITY_TAGS,
    );
}
