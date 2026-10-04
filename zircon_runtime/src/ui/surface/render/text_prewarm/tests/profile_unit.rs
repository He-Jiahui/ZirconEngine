use super::{
    record_compiled_rich_text_cache_profile, record_font_handle_registry_profile,
    record_text_layout_resolve_profile, CompiledRichTextCacheReport, FontHandleRegistryDelta,
    FontHandleRegistryReport, ShapedRunCacheDelta, ShapedRunCacheReport,
};
use crate::core::runtime::diagnostics::profiling::{
    reset_capture, snapshot, start_capture, test_capture_lock, ProfileCaptureConfig,
};
use crate::ui::text::UiTextMeasureCache;

use super::super::{prewarm_render_command_text, PendingOwnerTextLayouts};

#[test]
fn empty_render_command_prewarm_records_fixed_zero_counters() {
    let _capture_guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "ui-text-empty-prewarm-profile".to_string();
    config.max_spans = 4;
    config.max_counters = 16;
    start_capture(config);

    let mut cache = UiTextMeasureCache::default();
    cache.begin_frame();
    prewarm_render_command_text(&[], &PendingOwnerTextLayouts::default(), &mut cache);
    let profile = snapshot();
    reset_capture();

    for name in [
        "ui_text.prewarm.requested",
        "ui_text.prewarm.cache_hits",
        "ui_text.prewarm.cache_misses",
        "ui_text.prewarm.batch_duplicates",
        "ui_text.prewarm.shaped",
        "ui_text.prewarm.inserted",
        "ui_text.prewarm.invalid_requests",
        "ui_text.prewarm.generation_deferred",
        "ui_text.prewarm.failed",
        "ui_text.prewarm.caller_wait_nanos",
    ] {
        let counter = profile
            .counters
            .iter()
            .find(|counter| counter.stream == "runtime" && counter.name == name)
            .unwrap_or_else(|| panic!("empty prewarm stage omitted fixed counter: {name}"));
        assert_eq!(
            counter.value, 0.0,
            "empty prewarm counter must be zero: {name}"
        );
    }
}

#[test]
fn shaped_run_cache_delta_saturates_independent_counters() {
    let before = ShapedRunCacheReport {
        hit_count: 10,
        miss_count: 7,
        lookup_candidate_count: 13,
        owned_key_allocation_bytes: 17,
        eviction_scan_count: 19,
        entry_move_count: 23,
        insert_count: 5,
        evicted_count: 29,
        ..ShapedRunCacheReport::default()
    };
    let after = ShapedRunCacheReport {
        hit_count: 14,
        miss_count: 6,
        lookup_candidate_count: 19,
        owned_key_allocation_bytes: 31,
        eviction_scan_count: 37,
        entry_move_count: 41,
        insert_count: 8,
        evicted_count: 43,
        ..ShapedRunCacheReport::default()
    };

    assert_eq!(
        ShapedRunCacheDelta::between(before, after),
        ShapedRunCacheDelta {
            hit_count: 4,
            miss_count: 0,
            lookup_candidate_count: 6,
            owned_key_allocation_bytes: 14,
            eviction_scan_count: 18,
            entry_move_count: 18,
            insert_count: 3,
            evicted_count: 14,
        }
    );
}

#[test]
fn text_layout_profile_projects_the_effective_hard_line_budget() {
    let _capture_guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "ui-text-hard-line-budget-profile".to_string();
    config.max_counters = 128;
    start_capture(config);

    let mut cache = UiTextMeasureCache::default();
    cache.begin_frame();
    record_text_layout_resolve_profile(&cache, ShapedRunCacheReport::default(), 0);
    let profile = snapshot();
    reset_capture();

    for (name, value) in [
        ("text.runtime_budget.hard_line_cache_entries", 16.0),
        (
            "text.runtime_budget.hard_line_cache_bytes",
            (32 * 1024 * 1024) as f64,
        ),
        ("ui_text.hard_line_cache.resident_entries", 0.0),
        ("ui_text.hard_line_cache.resident_bytes", 0.0),
        ("ui_text.session.layout_fallbacks", 0.0),
        ("ui_text.session.shaping_failure_requests", 0.0),
        ("ui_text.session.backend_direct_runs", 0.0),
        ("ui_text.session.backend_alternate_runs", 0.0),
        ("ui_text.session.backend_hybrid_runs", 0.0),
        ("ui_text.session.shaping_attempts", 0.0),
        ("ui_text.session.font_resolution.primary_text_requests", 0.0),
        (
            "ui_text.session.font_resolution.decision_coverage_calls",
            0.0,
        ),
    ] {
        let counter = profile
            .counters
            .iter()
            .find(|counter| counter.stream == "runtime" && counter.name == name)
            .unwrap_or_else(|| panic!("hard-line budget counter missing: {name}"));
        assert_eq!(counter.value, value);
    }
}

#[test]
fn font_handle_frame_profile_projects_fixed_snapshot_deltas() {
    let _capture_guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "ui-text-font-handle-profile".to_string();
    config.max_counters = 16;
    start_capture(config);
    let before = FontHandleRegistryReport {
        registration_batch_count: 10,
        registration_lock_acquire_count: 9,
        registration_unique_pair_count: 11,
        resolution_batch_count: 8,
        resolution_snapshot_acquire_count: 7,
        resolution_unique_pair_count: 12,
        ..FontHandleRegistryReport::default()
    };
    let after = FontHandleRegistryReport {
        registration_batch_count: 13,
        registration_lock_acquire_count: 11,
        registration_unique_pair_count: 16,
        resolution_batch_count: 12,
        resolution_snapshot_acquire_count: 10,
        resolution_unique_pair_count: 18,
        ..before
    };

    record_font_handle_registry_profile(FontHandleRegistryDelta::between(before, after));
    let profile = snapshot();
    reset_capture();

    for (name, value) in [
        ("registration_batches", 3.0),
        ("registration_lock_acquires", 2.0),
        ("registration_unique_pairs", 5.0),
        ("resolution_batches", 4.0),
        ("resolution_snapshot_acquires", 3.0),
        ("resolution_unique_pairs", 6.0),
    ] {
        let full_name = format!("ui_text.font_handles.{name}");
        let counter = profile
            .counters
            .iter()
            .find(|counter| counter.stream == "runtime" && counter.name == full_name)
            .unwrap_or_else(|| panic!("font-handle frame counter missing: {full_name}"));
        assert_eq!(counter.value, value);
    }
    assert_eq!(
        profile
            .counters
            .iter()
            .filter(|counter| counter.name.starts_with("ui_text.font_handles."))
            .count(),
        13
    );
}

#[test]
fn compiled_rich_text_cache_profile_projects_fixed_frame_counters() {
    let _capture_guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "ui-text-compiled-rich-cache-profile".to_string();
    config.max_counters = 16;
    start_capture(config);

    {
        crate::profile_frame!("runtime", "ui_text.compiled_rich_cache_profile");
        record_compiled_rich_text_cache_profile(CompiledRichTextCacheReport {
            parser_identity: 29,
            decorator_generation: 31,
            emoji_generation: 37,
            compile_requests_in_flight: 41,
            single_flight_wait_count: 43,
            single_flight_wait_nanos: 47,
            single_flight_wait_max_nanos: 53,
            hit_count: 3,
            miss_count: 5,
            parse_count: 7,
            eviction_count: 11,
            admission_bypass_count: 13,
            candidate_probe_count: 17,
            resident_entries: 19,
            resident_bytes: 23,
            telemetry_saturated: true,
            ..CompiledRichTextCacheReport::default()
        });
    }
    let profile = snapshot();
    reset_capture();

    for (name, value) in [
        ("ui_text.rich_cache.parser_identity", 29.0),
        ("ui_text.rich_cache.decorator_generation", 31.0),
        ("ui_text.rich_cache.emoji_generation", 37.0),
        ("ui_text.rich_cache.compile_requests_in_flight", 41.0),
        ("ui_text.rich_cache.single_flight_waits", 43.0),
        ("ui_text.rich_cache.single_flight_wait_nanos", 47.0),
        ("ui_text.rich_cache.single_flight_wait_max_nanos", 53.0),
        ("ui_text.rich_cache.hits", 3.0),
        ("ui_text.rich_cache.misses", 5.0),
        ("ui_text.rich_cache.parses", 7.0),
        ("ui_text.rich_cache.evictions", 11.0),
        ("ui_text.rich_cache.admission_bypasses", 13.0),
        ("ui_text.rich_cache.lookup_candidates", 17.0),
        ("ui_text.rich_cache.resident_entries", 19.0),
        ("ui_text.rich_cache.resident_bytes", 23.0),
        ("ui_text.rich_cache.counter_saturated", 1.0),
    ] {
        let samples = profile
            .counters
            .iter()
            .filter(|counter| counter.stream == "runtime" && counter.name == name)
            .collect::<Vec<_>>();
        assert_eq!(samples.len(), 1, "missing or duplicate counter: {name}");
        assert_eq!(samples[0].value, value, "unexpected counter value: {name}");
        assert_eq!(samples[0].frame_index, Some(0));
    }
    assert_eq!(profile.counters.len(), 16);
}
