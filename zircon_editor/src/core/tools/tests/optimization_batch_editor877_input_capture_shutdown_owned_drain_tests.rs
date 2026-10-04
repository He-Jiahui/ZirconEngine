use std::collections::BTreeMap;
use std::hint::black_box;
use std::num::NonZeroU64;
use std::time::Instant;

use super::*;
use crate::core::editor_event::ViewInstanceId;

const CAPTURE_COUNT: usize = 4_096;
const SAMPLE_PAIRS: usize = 101;

#[test]
fn editor877_input_capture_shutdown_owned_drain_preserves_id_order() {
    let mut authority = ToolInputCaptureAuthority::new();
    let sources = [pointer_source(3), pointer_source(1), pointer_source(2)];
    let expected_ids = sources
        .iter()
        .enumerate()
        .map(|(index, source)| {
            captured_id(authority.begin(request(owner((index + 1) as u64), source.clone())))
        })
        .collect::<Vec<_>>();

    let report = authority.shutdown();
    assert_eq!(
        report
            .outcome()
            .iter()
            .map(ToolInputCaptureHandle::id)
            .collect::<Vec<_>>(),
        expected_ids
    );
    assert_eq!(
        report
            .events()
            .iter()
            .map(|event| match event {
                ToolInputCaptureEvent::Ended {
                    handle,
                    disposition: ToolInputCaptureDisposition::Shutdown,
                } => handle.id(),
                event => panic!("expected ordered shutdown event, got {event:?}"),
            })
            .collect::<Vec<_>>(),
        expected_ids
    );
    assert!(sources
        .iter()
        .all(|source| authority.active_for_source(source).is_none()));
    assert_eq!(authority.active().count(), 0);

    let source = include_str!("../input_capture.rs");
    let shutdown = source
        .split("pub(crate) fn shutdown")
        .nth(1)
        .expect("input capture shutdown owner must exist")
        .split("fn start_capture")
        .next()
        .expect("input capture shutdown owner must stay bounded");
    assert!(shutdown.contains("std::mem::take(&mut self.captures)"));
    assert!(!shutdown.contains("self.captures.keys().copied().collect"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor877_input_capture_shutdown_owned_drain_benchmark() {
    let captures = (0..CAPTURE_COUNT)
        .map(|index| (index, format!("editor877.capture.{index:05}.long.identity")))
        .collect::<BTreeMap<_, _>>();
    let by_source = (0..CAPTURE_COUNT)
        .map(|index| (index, index))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        legacy_shutdown_model(captures.clone(), by_source.clone()).0,
        CAPTURE_COUNT
    );
    assert_eq!(
        optimized_shutdown_model(captures.clone(), by_source.clone()).0,
        CAPTURE_COUNT
    );

    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let legacy_captures = captures.clone();
        let legacy_sources = by_source.clone();
        let optimized_captures = captures.clone();
        let optimized_sources = by_source.clone();
        if pair % 2 == 0 {
            legacy.push(measure(move || {
                legacy_shutdown_model(legacy_captures, legacy_sources)
            }));
            optimized.push(measure(move || {
                optimized_shutdown_model(optimized_captures, optimized_sources)
            }));
        } else {
            optimized.push(measure(move || {
                optimized_shutdown_model(optimized_captures, optimized_sources)
            }));
            legacy.push(measure(move || {
                legacy_shutdown_model(legacy_captures, legacy_sources)
            }));
        }
    }

    let legacy_p50 = percentile(&legacy, 50);
    let legacy_p95 = percentile(&legacy, 95);
    let legacy_p99 = percentile(&legacy, 99);
    let optimized_p50 = percentile(&optimized, 50);
    let optimized_p95 = percentile(&optimized, 95);
    let optimized_p99 = percentile(&optimized, 99);
    println!(
        "EDITOR877_INPUT_CAPTURE_SHUTDOWN_OWNED_DRAIN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} captures={CAPTURE_COUNT} legacy_id_slots={CAPTURE_COUNT} optimized_id_slots=0 legacy_tree_removals={} optimized_tree_removals=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99}",
        CAPTURE_COUNT * 2
    );
    assert!(
        optimized_p95 < legacy_p95,
        "expected owned drain model to improve P95, got legacy={legacy_p95}ns optimized={optimized_p95}ns"
    );
}

fn captured_id(report: ToolInputCaptureReport<ToolInputCaptureOutcome>) -> ToolInputCaptureId {
    match report.into_parts().0 {
        ToolInputCaptureOutcome::Captured { handle, .. } => handle.id(),
        outcome => panic!("expected captured input, got {outcome:?}"),
    }
}

fn owner(ordinal: u64) -> ToolInputCaptureOwner {
    ToolInputCaptureOwner::new(
        ToolLeaseId::from_ordinal(NonZeroU64::new(ordinal).expect("non-zero test lease")),
        ToolInstanceId::from_parts("editor877.tool", 1, ordinal).expect("valid test instance"),
    )
}

fn pointer_source(pointer: u64) -> ToolInputSource {
    ToolInputSource::Pointer {
        scope: ToolInputScope::new(
            UiWindowId::new("editor877.window"),
            UiSurfaceId::new("editor877.viewport"),
        ),
        user_id: None,
        device_id: None,
        pointer_id: Some(UiPointerId::new(pointer)),
        pointer_source: UiPointerSource::Mouse,
    }
}

fn request(owner: ToolInputCaptureOwner, source: ToolInputSource) -> ToolInputCaptureRequest {
    ToolInputCaptureRequest::new(
        owner,
        source,
        ToolResourceKey::viewport_input(ViewInstanceId::new("editor.scene#editor877")),
        ToolInputCapturePriority::new(1),
    )
}

fn legacy_shutdown_model(
    mut captures: BTreeMap<usize, String>,
    mut by_source: BTreeMap<usize, usize>,
) -> (usize, usize) {
    let ids = captures.keys().copied().collect::<Vec<_>>();
    let mut ended = Vec::with_capacity(ids.len());
    let mut events = Vec::with_capacity(ids.len());
    for id in ids {
        let handle = captures.remove(&id).expect("modeled capture must exist");
        by_source.remove(&id);
        events.push(handle.clone());
        ended.push(handle);
    }
    (black_box(ended).len(), black_box(events).len())
}

fn optimized_shutdown_model(
    captures: BTreeMap<usize, String>,
    mut by_source: BTreeMap<usize, usize>,
) -> (usize, usize) {
    by_source.clear();
    let mut ended = Vec::with_capacity(captures.len());
    let mut events = Vec::with_capacity(captures.len());
    for handle in captures.into_values() {
        events.push(handle.clone());
        ended.push(handle);
    }
    (black_box(ended).len(), black_box(events).len())
}

fn measure<T>(work: impl FnOnce() -> T) -> u128 {
    let started = Instant::now();
    black_box(work());
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
