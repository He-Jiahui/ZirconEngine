use super::super::super::super::shader_sources::POST_PROCESS_SHADER;

#[test]
fn scene_composite_shader_entry_is_split_from_uber() {
    assert!(POST_PROCESS_SHADER.contains("fn fs_scene_composite"));
    assert!(POST_PROCESS_SHADER.contains("apply_scene_composite"));
}
