use super::super::super::super::shader_sources::POST_PROCESS_SHADER;

#[test]
fn post_process_shader_exposes_split_ssr_resolve_entry_point() {
    assert!(POST_PROCESS_SHADER.contains("fn fs_screen_space_reflection_resolve"));
    assert!(POST_PROCESS_SHADER.contains("-> @location(0) vec4<f32>"));
    assert!(POST_PROCESS_SHADER.contains("resolve_screen_space_reflection_history"));
}
