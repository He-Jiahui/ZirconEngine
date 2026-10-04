use super::super::super::super::shader_sources::POST_PROCESS_SHADER;

#[test]
fn depth_of_field_shader_entry_is_split_from_uber() {
    assert!(POST_PROCESS_SHADER.contains("fn fs_depth_of_field"));
    assert!(POST_PROCESS_SHADER.contains("apply_effect_blur_family"));
}
