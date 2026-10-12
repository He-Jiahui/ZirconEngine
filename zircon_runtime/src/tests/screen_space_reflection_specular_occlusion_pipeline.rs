use super::super::super::super::shader_sources::POST_PROCESS_SHADER;

#[test]
fn post_process_shader_exposes_split_ssr_specular_occlusion_entry_point() {
    assert!(POST_PROCESS_SHADER.contains("fn fs_screen_space_reflection_specular_occlusion"));
    assert!(POST_PROCESS_SHADER.contains("resolve_screen_space_reflection_specular_occlusion"));
    assert!(POST_PROCESS_SHADER.contains("load_screen_space_reflection_specular_occlusion"));
}
