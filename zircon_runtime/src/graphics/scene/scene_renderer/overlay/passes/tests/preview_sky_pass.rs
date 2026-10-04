#[test]
fn disabled_sky_skips_volumetric_gpu_objects_before_recording() {
    let source = include_str!("../preview_sky_pass.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("preview sky implementation");
    let enabled_guard = implementation
        .find("let skybox_enabled = frame.environment().skybox.is_enabled()")
        .expect("skybox enabled guard");
    let params_buffer = implementation
        .find("volumetric_apply.create_params_buffer")
        .expect("volumetric params buffer creation");

    assert!(enabled_guard < params_buffer);
    assert!(implementation.contains("let volumetric_binding = skybox_enabled.then(||"));
    assert!(implementation.contains("if let Some((_params_buffer, bind_group))"));
}
