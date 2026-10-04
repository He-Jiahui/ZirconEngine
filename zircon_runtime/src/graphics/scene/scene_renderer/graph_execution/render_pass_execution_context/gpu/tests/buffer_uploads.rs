#[test]
fn temporal_and_bloom_passes_append_pre_submit_uploads_to_the_pass_context() {
    let temporal = include_str!("../post_process/temporal.rs");
    let effects = include_str!("../post_process/effects.rs");

    let taa = temporal
        .find("execute_taa_resolve(")
        .expect("TAA executor call");
    let velocity = temporal
        .find("execute_velocity_camera(")
        .expect("velocity executor call");
    assert!(temporal[taa..velocity].contains("self.append_pre_submit_buffer_uploads("));
    assert!(temporal[velocity..].contains("self.append_pre_submit_buffer_uploads("));

    let bloom = effects.find("execute_bloom(").expect("bloom executor call");
    assert!(effects[bloom..].contains("self.append_pre_submit_buffer_uploads("));
}
