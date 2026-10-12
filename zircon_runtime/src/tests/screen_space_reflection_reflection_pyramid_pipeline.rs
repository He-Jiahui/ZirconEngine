use super::super::super::super::shader_sources::POST_PROCESS_SHADER;

#[test]
fn post_process_shader_exposes_split_ssr_reflection_pyramid_entry_point() {
    assert!(POST_PROCESS_SHADER.contains("fn fs_screen_space_reflection_reflection_pyramid"));
    assert!(POST_PROCESS_SHADER.contains("resolve_screen_space_reflection_reflection_pyramid"));
    assert!(POST_PROCESS_SHADER.contains("screen_space_reflection_reflection_pyramid_tex"));
    assert!(POST_PROCESS_SHADER.contains("@group(0) @binding(24)"));
}
