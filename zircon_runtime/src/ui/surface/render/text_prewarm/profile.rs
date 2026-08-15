use crate::text::cache::ShapedRunCacheReport;
use crate::text::font::{font_handle_registry_report, FontHandleRegistryReport};
use crate::text::parallel::shape_pool::TextParallelShapeBatchReport;
use crate::text::CompiledRichTextCacheReport;
use crate::ui::text::UiTextMeasureCache;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ShapedRunCacheDelta {
    hit_count: u64,
    miss_count: u64,
    lookup_candidate_count: u64,
    owned_key_allocation_bytes: u64,
    eviction_scan_count: u64,
    entry_move_count: u64,
    insert_count: u64,
    evicted_count: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct FontHandleRegistryDelta {
    registration_batch_count: u64,
    registration_lock_acquire_count: u64,
    registration_lock_wait_nanos: u64,
    registration_lock_hold_nanos: u64,
    registration_snapshot_publish_count: u64,
    registration_unique_pair_count: u64,
    registration_rejected_pair_count: u64,
    resolution_batch_count: u64,
    resolution_snapshot_acquire_count: u64,
    resolution_snapshot_wait_nanos: u64,
    resolution_snapshot_hold_nanos: u64,
    resolution_unique_pair_count: u64,
    resolution_rejected_pair_count: u64,
}

impl FontHandleRegistryDelta {
    fn between(before: FontHandleRegistryReport, after: FontHandleRegistryReport) -> Self {
        Self {
            registration_batch_count: after
                .registration_batch_count
                .saturating_sub(before.registration_batch_count),
            registration_lock_acquire_count: after
                .registration_lock_acquire_count
                .saturating_sub(before.registration_lock_acquire_count),
            registration_lock_wait_nanos: after
                .registration_lock_wait_nanos
                .saturating_sub(before.registration_lock_wait_nanos),
            registration_lock_hold_nanos: after
                .registration_lock_hold_nanos
                .saturating_sub(before.registration_lock_hold_nanos),
            registration_snapshot_publish_count: after
                .registration_snapshot_publish_count
                .saturating_sub(before.registration_snapshot_publish_count),
            registration_unique_pair_count: after
                .registration_unique_pair_count
                .saturating_sub(before.registration_unique_pair_count),
            registration_rejected_pair_count: after
                .registration_rejected_pair_count
                .saturating_sub(before.registration_rejected_pair_count),
            resolution_batch_count: after
                .resolution_batch_count
                .saturating_sub(before.resolution_batch_count),
            resolution_snapshot_acquire_count: after
                .resolution_snapshot_acquire_count
                .saturating_sub(before.resolution_snapshot_acquire_count),
            resolution_snapshot_wait_nanos: after
                .resolution_snapshot_wait_nanos
                .saturating_sub(before.resolution_snapshot_wait_nanos),
            resolution_snapshot_hold_nanos: after
                .resolution_snapshot_hold_nanos
                .saturating_sub(before.resolution_snapshot_hold_nanos),
            resolution_unique_pair_count: after
                .resolution_unique_pair_count
                .saturating_sub(before.resolution_unique_pair_count),
            resolution_rejected_pair_count: after
                .resolution_rejected_pair_count
                .saturating_sub(before.resolution_rejected_pair_count),
        }
    }
}

pub(in crate::ui::surface::render) struct TextFontHandleFrameProfile {
    before: FontHandleRegistryReport,
}

impl TextFontHandleFrameProfile {
    pub(in crate::ui::surface::render) fn begin() -> Self {
        Self {
            before: font_handle_registry_report(),
        }
    }

    pub(in crate::ui::surface::render) fn finish(self) {
        record_font_handle_registry_profile(FontHandleRegistryDelta::between(
            self.before,
            font_handle_registry_report(),
        ));
    }
}

fn record_font_handle_registry_profile(report: FontHandleRegistryDelta) {
    for (name, value) in [
        (
            "ui_text.font_handles.registration_batches",
            report.registration_batch_count,
        ),
        (
            "ui_text.font_handles.registration_lock_acquires",
            report.registration_lock_acquire_count,
        ),
        (
            "ui_text.font_handles.registration_lock_wait_nanos",
            report.registration_lock_wait_nanos,
        ),
        (
            "ui_text.font_handles.registration_lock_hold_nanos",
            report.registration_lock_hold_nanos,
        ),
        (
            "ui_text.font_handles.registration_snapshot_publishes",
            report.registration_snapshot_publish_count,
        ),
        (
            "ui_text.font_handles.registration_unique_pairs",
            report.registration_unique_pair_count,
        ),
        (
            "ui_text.font_handles.registration_rejected_pairs",
            report.registration_rejected_pair_count,
        ),
        (
            "ui_text.font_handles.resolution_batches",
            report.resolution_batch_count,
        ),
        (
            "ui_text.font_handles.resolution_snapshot_acquires",
            report.resolution_snapshot_acquire_count,
        ),
        (
            "ui_text.font_handles.resolution_snapshot_wait_nanos",
            report.resolution_snapshot_wait_nanos,
        ),
        (
            "ui_text.font_handles.resolution_snapshot_hold_nanos",
            report.resolution_snapshot_hold_nanos,
        ),
        (
            "ui_text.font_handles.resolution_unique_pairs",
            report.resolution_unique_pair_count,
        ),
        (
            "ui_text.font_handles.resolution_rejected_pairs",
            report.resolution_rejected_pair_count,
        ),
    ] {
        crate::profile_counter!("runtime", name, value);
    }
}

impl ShapedRunCacheDelta {
    fn between(before: ShapedRunCacheReport, after: ShapedRunCacheReport) -> Self {
        Self {
            hit_count: after.hit_count.saturating_sub(before.hit_count),
            miss_count: after.miss_count.saturating_sub(before.miss_count),
            lookup_candidate_count: after
                .lookup_candidate_count
                .saturating_sub(before.lookup_candidate_count),
            owned_key_allocation_bytes: after
                .owned_key_allocation_bytes
                .saturating_sub(before.owned_key_allocation_bytes),
            eviction_scan_count: after
                .eviction_scan_count
                .saturating_sub(before.eviction_scan_count),
            entry_move_count: after
                .entry_move_count
                .saturating_sub(before.entry_move_count),
            insert_count: after.insert_count.saturating_sub(before.insert_count),
            evicted_count: after.evicted_count.saturating_sub(before.evicted_count),
        }
    }
}

pub(super) fn record_text_prewarm_profile(report: TextParallelShapeBatchReport) {
    crate::profile_counter!(
        "runtime",
        "ui_text.prewarm.requested",
        report.requested_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.prewarm.cache_hits",
        report.cache_hit_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.prewarm.cache_misses",
        report.cache_miss_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.prewarm.batch_duplicates",
        report.batch_duplicate_count
    );
    crate::profile_counter!("runtime", "ui_text.prewarm.shaped", report.shaped_count);
    crate::profile_counter!("runtime", "ui_text.prewarm.inserted", report.inserted_count);
    crate::profile_counter!(
        "runtime",
        "ui_text.prewarm.caller_wait_nanos",
        report.caller_wait_nanos
    );
}

pub(in crate::ui::surface::render) fn record_text_extract_profile(
    command_count: usize,
    owner_text_count: usize,
) {
    crate::profile_counter!("runtime", "ui_text.extract.commands", command_count);
    crate::profile_counter!("runtime", "ui_text.extract.owner_text", owner_text_count);
}

pub(in crate::ui::surface::render) fn record_compiled_rich_text_cache_profile(
    report: CompiledRichTextCacheReport,
) {
    for (name, value) in [
        ("ui_text.rich_cache.hits", report.hit_count),
        ("ui_text.rich_cache.misses", report.miss_count),
        ("ui_text.rich_cache.parses", report.parse_count),
        ("ui_text.rich_cache.evictions", report.eviction_count),
        (
            "ui_text.rich_cache.admission_bypasses",
            report.admission_bypass_count,
        ),
        (
            "ui_text.rich_cache.lookup_candidates",
            report.candidate_probe_count,
        ),
        (
            "ui_text.rich_cache.resident_entries",
            report.resident_entries as u64,
        ),
        (
            "ui_text.rich_cache.resident_bytes",
            report.resident_bytes as u64,
        ),
    ] {
        crate::profile_counter!("runtime", name, value);
    }
}

pub(super) fn record_text_layout_resolve_profile(
    text_measure_cache: &UiTextMeasureCache,
    shaped_run_cache_before: ShapedRunCacheReport,
    uncached_document_resolve_count: usize,
) {
    let layout = text_measure_cache.frame_layout_report();
    let dedup = text_measure_cache.frame_layout_dedup_report();
    let shape_cache = ShapedRunCacheDelta::between(
        shaped_run_cache_before,
        text_measure_cache.frame_shaped_run_report(),
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_resolve.cache_hits",
        layout.hit_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_resolve.cache_misses",
        layout.miss_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_resolve.cache_lookup_candidates",
        layout.lookup_candidate_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_resolve.cache_eviction_scans",
        layout.eviction_scan_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_resolve.cache_entry_moves",
        layout.entry_move_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_resolve.cache_evictions",
        layout.evicted_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_resolve.frame_dedup_hits",
        dedup.hit_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_resolve.frame_dedup_misses",
        dedup.miss_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_resolve.uncached_document_resolves",
        uncached_document_resolve_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_resolve.shape_cache_hits",
        shape_cache.hit_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_resolve.shape_cache_misses",
        shape_cache.miss_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_resolve.shape_cache_lookup_candidates",
        shape_cache.lookup_candidate_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_resolve.shape_cache_owned_key_allocation_bytes",
        shape_cache.owned_key_allocation_bytes
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_resolve.shape_cache_eviction_scans",
        shape_cache.eviction_scan_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_resolve.shape_cache_entry_moves",
        shape_cache.entry_move_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_resolve.shape_cache_inserts",
        shape_cache.insert_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.layout_resolve.shape_cache_evictions",
        shape_cache.evicted_count
    );
}

#[cfg(test)]
mod tests {
    use super::{
        record_compiled_rich_text_cache_profile, record_font_handle_registry_profile,
        CompiledRichTextCacheReport, FontHandleRegistryDelta, FontHandleRegistryReport,
        ShapedRunCacheDelta, ShapedRunCacheReport,
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
                hit_count: 3,
                miss_count: 5,
                parse_count: 7,
                eviction_count: 11,
                admission_bypass_count: 13,
                candidate_probe_count: 17,
                resident_entries: 19,
                resident_bytes: 23,
                ..CompiledRichTextCacheReport::default()
            });
        }
        let profile = snapshot();
        reset_capture();

        for (name, value) in [
            ("ui_text.rich_cache.hits", 3.0),
            ("ui_text.rich_cache.misses", 5.0),
            ("ui_text.rich_cache.parses", 7.0),
            ("ui_text.rich_cache.evictions", 11.0),
            ("ui_text.rich_cache.admission_bypasses", 13.0),
            ("ui_text.rich_cache.lookup_candidates", 17.0),
            ("ui_text.rich_cache.resident_entries", 19.0),
            ("ui_text.rich_cache.resident_bytes", 23.0),
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
        assert_eq!(profile.counters.len(), 8);
    }
}
