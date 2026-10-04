#[test]
fn coarse_pyramid_params_have_one_producer_outside_the_mip_loop() {
    let source = include_str!(
        "../../../../graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection.rs"
    );
    let method = source
        .split("fn record_screen_space_reflection_reflection_pyramid_coarse_to_resource")
        .nth(1)
        .expect("coarse-pyramid graph method")
        .split("fn record_screen_space_reflection_reflection_pyramid_to_resource")
        .next()
        .expect("coarse-pyramid graph method end");

    assert_eq!(
        method
            .matches("prepare_screen_space_reflection_reflection_pyramid_coarse_params(")
            .count(),
        1
    );
    assert!(method.contains("self.append_pre_submit_buffer_uploads("));
}
