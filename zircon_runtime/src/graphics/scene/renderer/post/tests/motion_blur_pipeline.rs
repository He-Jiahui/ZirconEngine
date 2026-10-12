use super::super::super::super::shader_sources::POST_PROCESS_SHADER;

#[test]
fn motion_blur_shader_entry_is_split_from_uber() {
    assert!(POST_PROCESS_SHADER.contains("fn fs_motion_blur"));
    assert!(POST_PROCESS_SHADER.contains("apply_motion_blur_vector_gather"));
}
