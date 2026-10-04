use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::event_ui::UiNodeId;

use super::*;

fn route_identity(index: u64) -> ScreenSpaceUiTextRouteIdentity {
    ScreenSpaceUiTextRouteIdentity::new(
        format!("runtime.ui.auto-route.{index:05}"),
        UiNodeId::new(index),
        None,
    )
}

fn route_entry(last_seen_frame: u64, recency_token: u64) -> AutoTextRasterRouteEntry {
    AutoTextRasterRouteEntry {
        command_generation: recency_token,
        mode: UiTextRenderMode::Native,
        last_seen_frame,
        recency_token,
    }
}

#[test]
fn optimization_batch_20260826h_runtime11c_recency_projection_preserves_frame_token_order() {
    let oldest = route_identity(7);
    let same_frame_earlier = route_identity(2);
    let same_frame_later = route_identity(9);
    let entries = [
        (same_frame_later.clone(), route_entry(11, 9)),
        (oldest.clone(), route_entry(3, 7)),
        (same_frame_earlier.clone(), route_entry(11, 4)),
    ]
    .into_iter()
    .collect::<HashMap<_, _>>();

    let compacted = compact_live_recency(&entries)
        .into_iter()
        .map(|recency| (recency.identity, recency.token))
        .collect::<Vec<_>>();

    assert_eq!(
        compacted,
        vec![(oldest, 7), (same_frame_earlier, 4), (same_frame_later, 9)]
    );
}

#[test]
fn optimization_batch_20260826h_runtime11c_recency_sort_uses_cached_projection() {
    let source = include_str!("../auto_route.rs");
    let compaction = source
        .split("fn compact_recency_if_needed")
        .nth(1)
        .expect("recency compaction method")
        .split("fn entry_matches_recency")
        .next()
        .expect("bounded recency compaction method");
    let projection = source
        .split("fn compact_live_recency")
        .nth(1)
        .expect("cached recency projection")
        .split("fn glyph_raster_path_for_mode")
        .next()
        .expect("bounded cached recency projection");

    assert!(compaction.contains("compact_live_recency(&self.entries)"));
    assert!(!compaction.contains(".get(&recency.identity)"));
    assert!(projection.contains("entry.last_seen_frame, entry.recency_token"));
    assert!(projection.contains("identity: identity.clone()"));
}

#[test]
fn active_cached_frame_route_survives_idle_eviction_without_per_frame_touch() {
    let identity = route_identity(17);
    let absent_identity = route_identity(18);
    let mut router = AutoTextRasterRouter::with_capacity_for_test(2);
    router.entries.insert(identity.clone(), route_entry(0, 1));
    router.recency.push_back(AutoTextRasterRouteRecency {
        identity: identity.clone(),
        token: 1,
    });
    router.replace_active_routes([identity.clone(), absent_identity]);
    assert_eq!(router.active_routes.len(), 1);

    for _ in 0..=AUTO_TEXT_ROUTE_MAX_IDLE_FRAMES {
        router.begin_frame();
    }

    assert!(router.entries.contains_key(&identity));
    assert_eq!(router.frame_report().idle_eviction_count, 0);

    router.clear_active_routes();
    router.begin_frame();

    assert!(!router.entries.contains_key(&identity));
    assert_eq!(router.frame_report().idle_eviction_count, 1);
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_20260826h_runtime11c_recency_sort_projection_performance_evidence() {
    fn legacy_compact(
        entries: &HashMap<ScreenSpaceUiTextRouteIdentity, AutoTextRasterRouteEntry>,
    ) -> VecDeque<AutoTextRasterRouteRecency> {
        let mut live = entries
            .iter()
            .map(|(identity, entry)| AutoTextRasterRouteRecency {
                identity: identity.clone(),
                token: entry.recency_token,
            })
            .collect::<Vec<_>>();
        live.sort_by_key(|recency| {
            entries
                .get(&recency.identity)
                .map(|entry| (entry.last_seen_frame, entry.recency_token))
                .unwrap_or_default()
        });
        live.into()
    }

    let entries = (0..4_096_u64)
        .rev()
        .map(|index| {
            (
                route_identity(index),
                route_entry(index % 257, index.saturating_add(1)),
            )
        })
        .collect::<HashMap<_, _>>();
    let mut legacy_samples = Vec::with_capacity(17);
    let mut projected_samples = Vec::with_capacity(17);
    for _ in 0..17 {
        let started = Instant::now();
        black_box(legacy_compact(black_box(&entries)));
        legacy_samples.push(started.elapsed().as_nanos());

        let started = Instant::now();
        black_box(compact_live_recency(black_box(&entries)));
        projected_samples.push(started.elapsed().as_nanos());
    }

    legacy_samples.sort_unstable();
    projected_samples.sort_unstable();
    let legacy_p95 = legacy_samples[16];
    let projected_p95 = projected_samples[16];
    println!(
        "RUNTIME11C_AUTO_TEXT_RECENCY_SORT_PROJECTION_BENCH_V1 entries={} legacy_p95_ns={} projected_p95_ns={} legacy_sort_map_lookup_path=1 projected_sort_map_lookup_path=0 identity_clones_before={} identity_clones_after={} target_ratio_bp=6000",
        entries.len(),
        legacy_p95,
        projected_p95,
        entries.len(),
        entries.len(),
    );
    assert!(
        projected_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
        "projected recency sort P95 {projected_p95} ns exceeded 60% of legacy {legacy_p95} ns"
    );
}
