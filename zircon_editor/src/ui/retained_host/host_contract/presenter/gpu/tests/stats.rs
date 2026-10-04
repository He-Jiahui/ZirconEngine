use super::*;

#[test]
fn present_stats_batch_preserves_region_and_optional_counter_semantics() {
    let mut counters = Vec::new();
    append_present_stats(&mut counters, &UiSurfacePresentStats::default(), true);

    assert_eq!(counters.len(), 52);
    assert_eq!(
        counters.last(),
        Some(&(UiPerfCounter::ChromeCommandPatchCount, 1.0))
    );
    assert!(!counters
        .iter()
        .any(|(counter, _)| *counter == UiPerfCounter::GpuTimestampSupportedPresentCount));
    assert!(!counters
        .iter()
        .any(|(counter, _)| *counter == UiPerfCounter::GpuTimeUs));
    assert!(!counters
        .iter()
        .any(|(counter, _)| *counter == UiPerfCounter::GpuProfileLatencyFrames));
}

#[test]
fn present_stats_batch_includes_supported_gpu_timing_once() {
    let mut stats = UiSurfacePresentStats::default();
    stats.gpu_timestamp_supported = true;
    stats.gpu_time_us = Some(125);
    stats.gpu_profile_latency_frames = 2;
    let mut counters = Vec::new();
    append_present_stats(&mut counters, &stats, false);

    assert_eq!(counters.len(), 55);
    assert_eq!(
        counters.last(),
        Some(&(UiPerfCounter::ChromeCommandFullRebuildCount, 1.0))
    );
    assert_eq!(
        counters
            .iter()
            .filter(|(counter, _)| { *counter == UiPerfCounter::GpuTimestampSupportedPresentCount })
            .count(),
        1
    );
    assert!(counters.contains(&(UiPerfCounter::GpuTimeUs, stats.gpu_time_us.unwrap() as f64)));
    assert!(counters.contains(&(
        UiPerfCounter::GpuProfileLatencyFrames,
        stats.gpu_profile_latency_frames as f64,
    )));
}

#[test]
fn present_stats_batch_includes_device_allocation_ledger_stats() {
    let mut stats = UiSurfacePresentStats::default();
    stats.image_device_allocation_count = 3;
    stats.image_device_allocation_bytes = 12_288;
    stats.image_registry_evicted_pinned_bytes = 4_096;
    stats.image_surface_pin_count = 5;
    stats.image_in_flight_present_pin_count = 2;
    stats.image_eviction_completion_count = 7;
    let mut counters = Vec::new();
    append_present_stats(&mut counters, &stats, true);

    assert!(counters.contains(&(UiPerfCounter::GpuImageDeviceAllocationCount, 3.0)));
    assert!(counters.contains(&(UiPerfCounter::GpuImageDeviceAllocationBytes, 12_288.0)));
    assert!(counters.contains(&(UiPerfCounter::GpuImageRegistryEvictedPinnedBytes, 4_096.0,)));
    assert!(counters.contains(&(UiPerfCounter::GpuImageSurfacePinCount, 5.0)));
    assert!(counters.contains(&(UiPerfCounter::GpuImageInFlightPresentPinCount, 2.0,)));
    assert!(counters.contains(&(UiPerfCounter::GpuImageEvictionCompletionCount, 7.0)));
}
