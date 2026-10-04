#[test]
fn resolved_gpu_timing_reports_do_not_borrow_the_native_device() {
    let caller = include_str!("../scene_renderer_realtime_ibl_diagnostics.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("realtime IBL diagnostic facade test boundary");
    let runtime = include_str!("../../environment/realtime_ibl_runtime.rs");
    let owner = runtime
        .split("pub(in crate::graphics) fn take_gpu_timing_reports(")
        .nth(1)
        .and_then(|source| {
            source
                .split("pub(in crate::graphics) fn take_cpu_timing_reports(")
                .next()
        })
        .expect("realtime IBL GPU timing report owner");

    assert!(caller.contains(".take_gpu_timing_reports()"));
    assert!(!caller.contains("backend.device"));
    assert!(!owner.contains("wgpu::Device"));
    assert!(owner.contains("timestamp_collector.take_completed()"));
}
