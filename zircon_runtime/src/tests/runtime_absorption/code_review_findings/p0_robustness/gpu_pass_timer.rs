#[test]
fn gpu_timer_readback_uses_the_shared_queue_without_a_private_map_lifecycle() {
    let timer = include_str!("../../../../rhi_wgpu/gpu_pass_timer.rs");
    let production = timer.split("\n#[cfg(test)]").next().unwrap_or_default();

    assert!(!production.contains(".expect("));
    assert!(!production.contains("map_async"));
    assert!(!production.contains("std::sync::mpsc"));
    assert!(!production.contains("readback_queue: GpuReadbackQueue"));
    assert!(production.contains("readback_queue: &mut GpuReadbackQueue"));
}
