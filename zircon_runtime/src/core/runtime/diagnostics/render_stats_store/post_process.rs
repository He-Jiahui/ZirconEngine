use crate::core::framework::render::{RenderColorLutReadbackReport, RenderStats};

use super::{record_bool, record_bytes, record_count, DiagnosticStore};

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

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_color_lut_readback(store, frame_index, stats.last_color_lut_readback_report);
}

fn record_color_lut_readback(
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

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{RenderColorLutReadbackReport, RenderStats};
    use crate::core::runtime::diagnostics::DiagnosticStore;

    use super::record;

    #[test]
    fn post_process_diagnostics_record_color_lut_readback_identity_report() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_color_lut_readback_report:
                RenderColorLutReadbackReport::from_raw_rgba16_float_identity_bytes(
                    [1, 1, 1],
                    &[0, 0, 0, 0, 0, 0, 0, 0x3c],
                ),
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(
            &store,
            "render.post_process.color_lut.readback.available",
            1.0,
            "bool",
            &["color_lut", "post_process", "readback", "render"],
        );
        assert_series(
            &store,
            "render.post_process.color_lut.readback.identity_within_epsilon",
            1.0,
            "bool",
            &[
                "color_lut",
                "identity",
                "post_process",
                "readback",
                "render",
            ],
        );
        assert_series(
            &store,
            "render.post_process.color_lut.readback.byte_len",
            8.0,
            "bytes",
            &["color_lut", "post_process", "readback", "render"],
        );
        assert_series(
            &store,
            "render.post_process.color_lut.readback.identity_max_abs_error_micro",
            0.0,
            "count",
            &[
                "color_lut",
                "identity",
                "post_process",
                "readback",
                "render",
            ],
        );
        assert_series(
            &store,
            "render.post_process.color_lut.readback.reference_kind",
            0.0,
            "count",
            &[
                "color_lut",
                "post_process",
                "readback",
                "reference",
                "render",
            ],
        );
        assert_series(
            &store,
            "render.post_process.color_lut.readback.reference_within_epsilon",
            1.0,
            "bool",
            &[
                "color_lut",
                "post_process",
                "readback",
                "reference",
                "render",
            ],
        );
    }

    #[test]
    fn post_process_diagnostics_record_color_lut_readback_user_lut_reference_report() {
        let mut store = DiagnosticStore::default();
        let stats = RenderStats {
            submitted_frames: 12,
            last_color_lut_readback_report:
                RenderColorLutReadbackReport::from_raw_rgba16_float_user_lut_bytes(
                    [1, 1, 1],
                    &[0, 0, 0, 0, 0, 0, 0, 0x3c],
                    |_| [0.0, 0.0, 0.0],
                ),
            ..RenderStats::default()
        };

        record(&mut store, &stats);

        assert_series(
            &store,
            "render.post_process.color_lut.readback.identity_within_epsilon",
            0.0,
            "bool",
            &[
                "color_lut",
                "identity",
                "post_process",
                "readback",
                "render",
            ],
        );
        assert_series(
            &store,
            "render.post_process.color_lut.readback.reference_kind",
            1.0,
            "count",
            &[
                "color_lut",
                "post_process",
                "readback",
                "reference",
                "render",
            ],
        );
        assert_series(
            &store,
            "render.post_process.color_lut.readback.reference_within_epsilon",
            1.0,
            "bool",
            &[
                "color_lut",
                "post_process",
                "readback",
                "reference",
                "render",
            ],
        );
        assert_series(
            &store,
            "render.post_process.color_lut.readback.reference_out_of_tolerance_sample_count",
            0.0,
            "count",
            &[
                "color_lut",
                "post_process",
                "readback",
                "reference",
                "render",
            ],
        );
    }

    fn assert_series(store: &DiagnosticStore, path: &str, value: f64, unit: &str, tags: &[&str]) {
        let snapshot = store.snapshot();
        let series = snapshot
            .series
            .iter()
            .find(|series| series.path.as_str() == path)
            .unwrap_or_else(|| panic!("missing diagnostic series `{path}`"));
        assert_eq!(series.current, Some(value));
        assert_eq!(series.unit.as_deref(), Some(unit));
        assert_eq!(
            series.subsystem_tags,
            tags.iter().map(|tag| tag.to_string()).collect::<Vec<_>>()
        );
        assert_eq!(series.history.len(), 1);
        assert_eq!(series.history[0].frame_index, 12);
        assert_eq!(series.history[0].value, value);
    }
}
