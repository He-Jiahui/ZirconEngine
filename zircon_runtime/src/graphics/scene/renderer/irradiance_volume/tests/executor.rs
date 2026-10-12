#[test]
fn irradiance_bind_executor_does_not_repeat_frame_preparation() {
    let production = include_str!("../executor.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("irradiance executor test boundary");

    assert!(production.contains("context.require_gpu()?"));
    assert!(!production.contains("select_irradiance_volume_for_view"));
    assert!(!production.contains("irradiance_volume_texture"));
    assert!(!production.contains(".prepare("));
    assert!(!production.contains("gpu.queue"));
}
