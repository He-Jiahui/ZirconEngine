use crate::core::framework::render::{
    RenderColorLutReadbackReport, RenderExposureReadbackReport, RenderStats,
};
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

#[test]
fn post_process_diagnostics_record_exposure_readback_report() {
    let mut store = DiagnosticStore::default();
    let stats = RenderStats {
        submitted_frames: 12,
        last_exposure_readback_report: RenderExposureReadbackReport::from_words([
            1.25, 9.5, 9.5, 1.0,
        ]),
        ..RenderStats::default()
    };

    record(&mut store, &stats);

    assert_series(
        &store,
        "render.post_process.exposure.readback.available",
        1.0,
        "bool",
        &["exposure", "post_process", "readback", "render"],
    );
    assert_series(
        &store,
        "render.post_process.exposure.readback.history_valid",
        1.0,
        "bool",
        &["exposure", "post_process", "readback", "render"],
    );
    assert_series(
        &store,
        "render.post_process.exposure.readback.byte_len",
        16.0,
        "bytes",
        &["exposure", "post_process", "readback", "render"],
    );
    assert_series(
        &store,
        "render.post_process.exposure.readback.invalid_word_count",
        0.0,
        "count",
        &["exposure", "post_process", "readback", "render"],
    );
    assert_series(
        &store,
        "render.post_process.exposure.readback.multiplier_micro",
        1_250_000.0,
        "count",
        &["exposure", "post_process", "readback", "render"],
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
