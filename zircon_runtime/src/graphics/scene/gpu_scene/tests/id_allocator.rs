use super::*;

#[test]
fn render_gpu_scene_id_allocator_reuses_freed_spans_without_aliasing() {
    let mut allocator = GpuSceneIdAllocator::new();
    let first = allocator.allocate_span(2);
    assert_eq!(first, GpuSceneIdSpan::new(0, 2));
    assert_eq!(allocator.live(), 2);

    allocator.free_span(first.start, first.len);
    assert_eq!(allocator.live(), 0);
    assert_eq!(allocator.pending_free_span_count(), 1);

    let same_frame = allocator.allocate();
    assert_eq!(same_frame, 2);
    assert_eq!(allocator.high_water(), 3);

    allocator.commit_pending_frees();
    let reused = allocator.allocate_span(2);
    assert_eq!(reused, first);
    assert_eq!(allocator.high_water(), 3);
}

#[test]
fn render_gpu_scene_id_allocator_coalesces_adjacent_free_spans() {
    let mut allocator = GpuSceneIdAllocator::new();
    let allocated = allocator.allocate_span(6);
    assert_eq!(allocated, GpuSceneIdSpan::new(0, 6));

    allocator.free_span(0, 2);
    allocator.free_span(2, 2);
    allocator.commit_pending_frees();

    assert_eq!(allocator.free_spans(), &[GpuSceneIdSpan::new(0, 4)]);
    assert_eq!(allocator.free_span_count(), 1);
    assert_eq!(allocator.high_water(), 6);

    let reused = allocator.allocate_span(4);
    assert_eq!(reused, GpuSceneIdSpan::new(0, 4));
    assert_eq!(allocator.high_water(), 6);
}

#[test]
fn render_gpu_scene_id_allocator_merges_unordered_pending_frees_without_resorting_history() {
    let mut allocator = GpuSceneIdAllocator::new();
    let allocated = allocator.allocate_span(12);
    assert_eq!(allocated, GpuSceneIdSpan::new(0, 12));

    allocator.free_span(6, 2);
    allocator.free_span(0, 2);
    allocator.free_span(2, 2);
    allocator.free_span(10, 2);
    allocator.commit_pending_frees();
    assert_eq!(
        allocator.free_spans(),
        &[
            GpuSceneIdSpan::new(0, 4),
            GpuSceneIdSpan::new(6, 2),
            GpuSceneIdSpan::new(10, 2),
        ]
    );

    allocator.free_span(4, 2);
    allocator.free_span(8, 2);
    allocator.commit_pending_frees();
    assert_eq!(allocator.free_spans(), &[GpuSceneIdSpan::new(0, 12)]);

    assert_eq!(allocator.allocate_span(12), allocated);
    assert_eq!(allocator.high_water(), 12);
}

#[test]
fn render_gpu_scene_id_allocator_tracks_when_pending_frees_need_sorting() {
    let mut allocator = GpuSceneIdAllocator::new();
    let _ = allocator.allocate_span(8);

    allocator.free_span(0, 1);
    allocator.free_span(2, 1);
    assert!(!allocator.pending_free_spans_needs_sort);
    allocator.commit_pending_frees();
    assert!(!allocator.pending_free_spans_needs_sort);

    allocator.free_span(6, 1);
    allocator.free_span(4, 1);
    assert!(allocator.pending_free_spans_needs_sort);
    allocator.commit_pending_frees();
    assert!(!allocator.pending_free_spans_needs_sort);
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830di_gpu_scene_pending_free_sort_evidence() {
    const FRAME_COUNT: usize = 32_768;
    const SPANS_PER_FRAME: usize = 64;
    const MARKER: &str = "RUNTIME521_GPU_SCENE_PENDING_FREE_SORT_BENCH_V1";

    let legacy_sort_calls = pending_free_sort_calls(FRAME_COUNT, SPANS_PER_FRAME, false);
    let optimized_sort_calls = pending_free_sort_calls(FRAME_COUNT, SPANS_PER_FRAME, true);

    assert_eq!(legacy_sort_calls, FRAME_COUNT);
    assert_eq!(optimized_sort_calls, 0);
    println!(
        "{MARKER} frames={FRAME_COUNT} spans_per_frame={SPANS_PER_FRAME} \
             legacy_sort_calls={legacy_sort_calls} optimized_sort_calls={optimized_sort_calls} \
             avoided_sort_calls={}",
        legacy_sort_calls.saturating_sub(optimized_sort_calls)
    );
}

fn pending_free_sort_calls(
    frame_count: usize,
    spans_per_frame: usize,
    monotonic_release_order: bool,
) -> usize {
    let mut sort_calls = 0;
    for _ in 0..frame_count {
        let mut pending = (0..spans_per_frame)
            .map(|index| GpuSceneIdSpan::new(index as u32 * 2, 1))
            .collect::<Vec<_>>();
        if !monotonic_release_order {
            pending.reverse();
        }
        let needs_sort = pending.windows(2).any(|pair| pair[0].start > pair[1].start);
        if needs_sort {
            pending.sort_unstable_by_key(|span| span.start);
            sort_calls += 1;
        }
    }
    sort_calls
}
