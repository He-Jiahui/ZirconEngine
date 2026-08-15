#[test]
fn editor_manager_keeps_raw_level_construction_inside_the_editor_crate() {
    let source = include_str!("../../ui/host/editor_manager_project.rs");

    assert!(
        source.contains("pub(crate) fn create_runtime_level("),
        "EditorManager must keep raw LevelSystem construction crate-private"
    );
    assert!(
        !source.contains("pub fn create_runtime_level("),
        "EditorManager must not expose raw LevelSystem construction as public API"
    );
}
