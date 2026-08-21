#![cfg(feature = "profiling")]

use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiTextOverflow, UiTextWrap},
};

use super::super::layout_profile_metrics_enabled;
use super::{layout_text, test_style};

#[test]
fn plain_layout_reports_pre_artifact_shaped_cache_deltas() {
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
    config.session_id = "plain-layout-pre-artifact-cache-deltas".to_owned();
    config.max_spans = 64;
    config.max_counters = 16;
    assert!(crate::core::diagnostics::profiling::start_capture(config).active);

    let layout = layout_text(
        "Measured wrapping keeps layout and artifact telemetry distinct.",
        &test_style(UiTextWrap::Word, UiTextOverflow::Clip),
        UiFrame::new(0.0, 0.0, 55.0, 120.0),
        None,
    );

    let snapshot = crate::core::diagnostics::profiling::snapshot();
    assert!(
        !crate::core::diagnostics::profiling::reset_capture().active,
        "layout profiling capture must reset before another test starts"
    );
    assert!(
        layout.lines.len() > 1,
        "sample must exercise wrapped plain text"
    );
    for (category, name) in [
        ("text.layout", "resolve_without_artifact"),
        ("text.artifact", "build_resolved_text_glyph_artifact"),
    ] {
        assert_eq!(
            snapshot
                .spans
                .iter()
                .filter(|span| span.category == category && span.name == name)
                .count(),
            1,
            "plain layout must expose exactly one {category}/{name} span"
        );
    }
    for name in [
        "layout_pre_artifact_shaped_cache_hit_count",
        "layout_pre_artifact_shaped_cache_miss_count",
    ] {
        let value = snapshot
            .counters
            .iter()
            .find(|counter| counter.stream == "runtime" && counter.name == name)
            .map(|counter| counter.value)
            .expect("layout must report its pre-artifact cache delta");
        assert!(
            value.is_finite() && value >= 0.0,
            "layout must report a finite non-negative {name}"
        );
    }
}

#[cfg(not(feature = "profiling-tracy"))]
#[test]
fn plain_layout_idle_cpu_profiler_skips_cache_measurement() {
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    assert!(
        !crate::core::diagnostics::profiling::capture_active(),
        "the idle-profiler contract requires capture to be inactive"
    );
    assert!(
        !layout_profile_metrics_enabled(),
        "idle CPU profiling must not read cache reports around every layout request"
    );
}
