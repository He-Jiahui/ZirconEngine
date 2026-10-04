#[test]
fn specular_occlusion_params_are_returned_as_pre_submit_uploads() {
    let source = include_str!("../execute_screen_space_reflection_specular_occlusion.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("specular-occlusion source");

    assert!(!production.contains("queue.write_buffer"));
    assert!(!production.contains("create_post_process_params_buffer"));
    assert!(production.contains("post_process_params_upload("));
}
