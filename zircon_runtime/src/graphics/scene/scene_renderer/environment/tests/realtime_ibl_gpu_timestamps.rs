use super::*;

#[test]
fn timestamp_pair_decodes_little_endian_words() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&40_u64.to_le_bytes());
    bytes.extend_from_slice(&58_u64.to_le_bytes());

    assert_eq!(decode_timestamp_pair(&bytes), Some([40, 58]));
}

#[test]
fn timestamp_delta_uses_queue_period_in_nanoseconds() {
    assert_eq!(elapsed_gpu_nanoseconds([100, 132], 2.5), 80.0);
}

#[test]
fn timestamp_delta_saturates_invalid_reverse_order() {
    assert_eq!(elapsed_gpu_nanoseconds([132, 100], 2.5), 0.0);
}

#[test]
fn gpu_timing_contract_excludes_cpu_recording_windows() {
    let source = include_str!("../realtime_ibl_gpu_timestamps.rs");
    let gpu_report = source
        .split("pub struct RealtimeIblGpuTimingReport {")
        .nth(1)
        .and_then(|definition| definition.split("\n}\n\n#[derive(Clone, Debug)]").next())
        .expect("GPU timing report definition");
    let gpu_metadata = source
        .split("pub(in crate::graphics) struct RealtimeIblGpuTimingMetadata {")
        .nth(1)
        .and_then(|definition| {
            definition
                .split("\n}\n\nimpl RealtimeIblGpuTimingMetadata")
                .next()
        })
        .expect("GPU timing metadata definition");

    for cpu_window in [
        "command_plan_creation_micros",
        "pipeline_ensure_micros",
        "binding_creation_micros",
        "capture_binding_creation_micros",
        "source_mip_binding_creation_micros",
    ] {
        assert!(
            !gpu_report.contains(cpu_window),
            "GPU timing report must not expose CPU recording window {cpu_window}"
        );
        assert!(
            !gpu_metadata.contains(cpu_window),
            "GPU timing metadata must not transport CPU recording window {cpu_window}"
        );
    }
}
