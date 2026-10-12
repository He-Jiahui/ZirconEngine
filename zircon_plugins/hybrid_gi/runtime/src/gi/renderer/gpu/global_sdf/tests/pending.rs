use super::*;

#[test]
fn empty_gpu_dispatch_retains_terminal_fallback_statistics() {
    let stats = GlobalSdfGpuBuildStats {
        candidate_overflow_page_count: 1,
        deferred_page_count: 3,
        ..GlobalSdfGpuBuildStats::default()
    };

    let dispatch = GlobalSdfGpuBuildDispatch::without_pending(stats);

    assert_eq!(dispatch.stats(), stats);
    assert!(!dispatch.encoded_gpu_work());
    assert!(dispatch.into_pending().is_none());
}
