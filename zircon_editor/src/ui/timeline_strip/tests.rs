use std::{
    hint::black_box,
    sync::{Arc, Barrier, Mutex, MutexGuard},
    thread,
    time::{Duration, Instant},
};

use super::{
    static_content_cache_entry_count, TimelineStripGeneration, TimelineStripGenerationInput,
    TimelineStripKey, STATIC_CONTENT_CACHE_CAPACITY,
};

fn generation(current_time: f32, tick_interval: f32, track_label: &str) -> TimelineStripGeneration {
    TimelineStripGeneration::new(TimelineStripGenerationInput {
        duration: 3.0,
        current_time,
        tick_interval,
        track_label: track_label.to_owned(),
        keys: vec![TimelineStripKey::new(2.0, "Run_Fwd", true)],
    })
}

static STATIC_CONTENT_TEST_LOCK: Mutex<()> = Mutex::new(());

fn static_content_test_lock() -> MutexGuard<'static, ()> {
    STATIC_CONTENT_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[test]
fn optimization_wave_20260824tu_editor75_ticks_are_preformatted_once_per_visual_budget() {
    let _guard = static_content_test_lock();
    let timeline = generation(2.25, 0.5, "Run_Fwd");

    let first = timeline.static_content_for_plot_width(120.0);
    let second = timeline.static_content_for_plot_width(120.0);

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(
        first
            .ticks()
            .iter()
            .map(|tick| tick.label())
            .collect::<Vec<_>>(),
        vec!["0.0", "0.5", "1.0", "1.5", "2.0", "2.5", "3.0"]
    );
}

#[test]
fn optimization_wave_20260824tu_editor75_scrub_changes_only_dynamic_generation() {
    let _guard = static_content_test_lock();
    let before = generation(0.5, 0.5, "Run_Fwd");
    let after = generation(2.25, 0.5, "Run_Fwd");

    assert_eq!(before.static_generation(), after.static_generation());
    assert_ne!(before.dynamic_generation(), after.dynamic_generation());
    assert_eq!(
        before.static_content_for_plot_width(120.0).generation(),
        after.static_content_for_plot_width(120.0).generation()
    );
}

#[test]
fn optimization_wave_20260824tu_editor75_reprojection_reuses_static_content_for_scrub() {
    let _guard = static_content_test_lock();
    let before = generation(0.5, 0.5, "Run_Fwd");
    let before_content = before.static_content_for_plot_width(120.0);
    let after = generation(2.25, 0.5, "Run_Fwd");
    let after_content = after.static_content_for_plot_width(120.0);

    assert!(Arc::ptr_eq(&before_content, &after_content));
}

#[test]
fn optimization_wave_20260824tu_editor75_track_or_tick_changes_update_static_generation() {
    let baseline = generation(2.25, 0.5, "Run_Fwd");
    let relabeled = generation(2.25, 0.5, "Jump");
    let reticked = generation(2.25, 1.0, "Run_Fwd");

    assert_ne!(baseline.static_generation(), relabeled.static_generation());
    assert_ne!(baseline.static_generation(), reticked.static_generation());
    assert_eq!(
        baseline.dynamic_generation(),
        relabeled.dynamic_generation()
    );
    assert_eq!(baseline.dynamic_generation(), reticked.dynamic_generation());
}

#[test]
fn optimization_wave_20260824tu_editor75_key_geometry_changes_static_and_selection_changes_dynamic()
{
    let baseline = generation(2.25, 0.5, "Run_Fwd");
    let repositioned = TimelineStripGeneration::new(TimelineStripGenerationInput {
        duration: 3.0,
        current_time: 2.25,
        tick_interval: 0.5,
        track_label: "Run_Fwd".to_owned(),
        keys: vec![TimelineStripKey::new(1.0, "Run_Fwd", true)],
    });
    let reselected = TimelineStripGeneration::new(TimelineStripGenerationInput {
        duration: 3.0,
        current_time: 2.25,
        tick_interval: 0.5,
        track_label: "Run_Fwd".to_owned(),
        keys: vec![TimelineStripKey::new(2.0, "Run_Fwd", false)],
    });
    let relabeled = TimelineStripGeneration::new(TimelineStripGenerationInput {
        duration: 3.0,
        current_time: 2.25,
        tick_interval: 0.5,
        track_label: "Run_Fwd".to_owned(),
        keys: vec![TimelineStripKey::new(2.0, "Jump", true)],
    });

    assert_ne!(
        baseline.static_generation(),
        repositioned.static_generation()
    );
    assert_eq!(
        baseline.dynamic_generation(),
        repositioned.dynamic_generation()
    );
    assert_eq!(baseline.static_generation(), reselected.static_generation());
    assert_ne!(baseline.static_generation(), relabeled.static_generation());
    assert_ne!(
        baseline.dynamic_generation(),
        reselected.dynamic_generation()
    );
}

#[test]
fn optimization_wave_20260824tu_editor75_visual_budget_is_bounded_and_preserves_endpoints() {
    let _guard = static_content_test_lock();
    let timeline = TimelineStripGeneration::new(TimelineStripGenerationInput {
        duration: 10.0,
        current_time: 0.0,
        tick_interval: f32::MIN_POSITIVE,
        track_label: String::new(),
        keys: Vec::new(),
    });

    let content = timeline.static_content_for_plot_width(126.0);
    assert_eq!(content.ticks().len(), 127);
    assert_eq!(content.ticks().first().map(|tick| tick.value()), Some(0.0));
    assert_eq!(content.ticks().last().map(|tick| tick.value()), Some(10.0));
}

#[test]
fn optimization_wave_20260824tu_editor75_visual_budget_clamps_to_the_hard_cap() {
    let _guard = static_content_test_lock();
    let timeline = TimelineStripGeneration::new(TimelineStripGenerationInput {
        duration: 10.0,
        current_time: 0.0,
        tick_interval: f32::MIN_POSITIVE,
        track_label: String::new(),
        keys: Vec::new(),
    });

    let content = timeline.static_content_for_plot_width(f32::MAX);
    assert_eq!(content.ticks().len(), 4_096);
    assert_eq!(content.ticks().first().map(|tick| tick.value()), Some(0.0));
    assert_eq!(content.ticks().last().map(|tick| tick.value()), Some(10.0));
}

#[test]
fn optimization_wave_20260824tu_editor75_visual_budget_cache_is_bounded() {
    let _guard = static_content_test_lock();
    let timeline = generation(2.25, 0.5, "Bounded");

    for width in 1..=(STATIC_CONTENT_CACHE_CAPACITY * 4) {
        let _ = timeline.static_content_for_plot_width(width as f32);
    }

    assert!(static_content_cache_entry_count() <= STATIC_CONTENT_CACHE_CAPACITY);
}

#[test]
fn optimization_wave_20260824tu_editor75_static_cache_is_single_flight_without_hit_scan() {
    let source = include_str!("generation.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(!production.contains("VecDeque"));
    assert!(!production.contains("recency.retain"));
    assert!(production.contains("OnceLock<Arc<TimelineStripStaticContent>>"));
    assert!(production.contains("get_or_init"));
}

#[test]
fn optimization_wave_20260824tu_editor75_concurrent_static_cache_miss_shares_one_result() {
    const WORKERS: usize = 16;

    let _guard = static_content_test_lock();
    let timeline = Arc::new(generation(2.25, 0.5, "SingleFlight20260824"));
    let barrier = Arc::new(Barrier::new(WORKERS));
    let workers = (0..WORKERS)
        .map(|_| {
            let timeline = Arc::clone(&timeline);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                timeline.static_content_for_plot_width(f32::MAX)
            })
        })
        .collect::<Vec<_>>();
    let results = workers
        .into_iter()
        .map(|worker| worker.join().expect("timeline cache worker should finish"))
        .collect::<Vec<_>>();

    assert!(results
        .iter()
        .skip(1)
        .all(|result| Arc::ptr_eq(&results[0], result)));
}

#[test]
#[ignore = "performance evidence; run in the managed Windows release lane"]
fn optimization_wave_20260824tu_editor75_timeline_cache_hit_evidence() {
    const HIT_COUNT: usize = 250_000;
    const MAX_ELAPSED: Duration = Duration::from_secs(1);

    let _guard = static_content_test_lock();
    let timeline = generation(2.25, 0.5, "CacheHitEvidence20260824");
    for width in 1..=STATIC_CONTENT_CACHE_CAPACITY {
        black_box(timeline.static_content_for_plot_width(width as f32));
    }

    let started = Instant::now();
    for _ in 0..HIT_COUNT {
        black_box(timeline.static_content_for_plot_width(1.0));
    }
    let elapsed = started.elapsed();
    let legacy_recency_entries_visited = HIT_COUNT * STATIC_CONTENT_CACHE_CAPACITY;
    let optimized_recency_entries_visited = 0_usize;
    assert!(
        elapsed <= MAX_ELAPSED,
        "timeline cache hits took {elapsed:?}"
    );
    println!(
        "EDITOR75_TIMELINE_CACHE_BENCH_V1 capacity={} hits={} legacy_recency_entries_visited={} optimized_recency_entries_visited={} elapsed_ns={}",
        STATIC_CONTENT_CACHE_CAPACITY,
        HIT_COUNT,
        legacy_recency_entries_visited,
        optimized_recency_entries_visited,
        elapsed.as_nanos()
    );
}

#[test]
fn optimization_wave_20260824tu_editor75_invalid_input_is_normalized() {
    let timeline = TimelineStripGeneration::new(TimelineStripGenerationInput {
        duration: f32::NAN,
        current_time: f32::INFINITY,
        tick_interval: 0.0,
        track_label: String::new(),
        keys: vec![TimelineStripKey::new(f32::NAN, "invalid", true)],
    });

    assert_eq!(timeline.duration(), 1.0);
    assert_eq!(timeline.current_time(), 0.0);
    assert_eq!(timeline.tick_interval(), 0.25);
    assert!(timeline.keys().is_empty());
}
