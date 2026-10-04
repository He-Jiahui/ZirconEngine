const SOURCE: &str = include_str!("../environment_capture_residency.rs");

fn production_source() -> &'static str {
    SOURCE
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("environment capture residency must retain a test boundary")
}

#[test]
fn residency_is_bounded_and_replaces_one_capture_id_atomically() {
    let source = production_source();
    let epoch_advance = source
        .find("self.observation_epoch = self.observation_epoch.saturating_add(1);")
        .expect("publication must advance the observation epoch once");
    let identity_read = source
        .find("let capture_id = output.identity().capture_id().to_string();")
        .expect("publication must resolve the resident capture identity");

    assert!(source.contains("MAX_RESIDENT_ENVIRONMENT_CAPTURES"));
    assert!(source.contains("HashMap<String, EnvironmentCaptureResidentOutput>"));
    assert!(source.contains("VecDeque<String>"));
    assert!(source.contains("fn publish("));
    assert!(source.contains("self.outputs.insert("));
    assert!(source.contains("self.outputs.remove("));
    assert!(epoch_advance < identity_read);
}

#[test]
fn residency_reports_exact_filtered_gpu_bytes_without_capture_scratch() {
    let source = production_source();

    assert!(source.contains("resident_gpu_bytes"));
    assert!(source.contains("fn observation_epoch(&self) -> u64"));
    assert!(source.contains("output.gpu_bytes()"));
    assert!(!source.contains("EnvironmentCaptureGpuTarget"));
    assert!(!source.contains("source_texture"));
    assert!(!source.contains("depth_texture"));
}
