#[test]
fn editor_manager_keeps_raw_scene_persistence_inside_the_editor_crate() {
    let source = include_str!("../../ui/host/editor_manager_project.rs");

    assert!(
        source.contains("pub(crate) fn save_project("),
        "EditorManager must keep raw Scene persistence crate-private"
    );
    assert!(
        !source.contains("pub fn save_project("),
        "EditorManager must not expose raw Scene persistence as public API"
    );
}
