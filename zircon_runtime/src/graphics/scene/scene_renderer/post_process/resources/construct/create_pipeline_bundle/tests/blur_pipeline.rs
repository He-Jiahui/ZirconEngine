use super::super::super::super::shader_sources::POST_PROCESS_SHADER;

#[test]
fn blur_shader_entry_is_split_from_uber() {
    assert!(POST_PROCESS_SHADER.contains("fn fs_blur"));
    assert!(POST_PROCESS_SHADER.contains("apply_effect_blur_family"));
}
