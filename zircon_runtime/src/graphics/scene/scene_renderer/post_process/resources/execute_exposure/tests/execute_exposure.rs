#[test]
fn exposure_params_have_one_frame_preparation_owner() {
    let source = include_str!("../execute_exposure.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("exposure production source");

    assert_eq!(production.matches("ExposureParams::new(").count(), 1);
    assert_eq!(
        production.matches("prepare_exposure_params_upload").count(),
        1
    );
    assert!(!production.contains("queue.write_buffer"));
    assert!(production.contains("WgpuBufferUpload::from_bytes("));
    assert!(!production.contains("EXPOSURE_ADAPTATION_DELTA_SECONDS"));
    assert!(production.contains("raw_real_delta_seconds"));
}
