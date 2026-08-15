#[test]
fn keyboard_dispatch_resolves_the_current_authority_keymap() {
    let source = include_str!("input_dispatch.rs");
    let reparsing = ["EditorKeymap::", "default_workbench()"].concat();

    assert!(!source.contains(&reparsing));
    assert!(source.contains("manager.resolve_keyboard_input(keyboard)"));
    assert!(!source.contains("self.keymap().resolve_keyboard_input"));
}

#[test]
fn command_adapter_mismatch_has_no_production_panic_path() {
    let source = include_str!("component_dispatch.rs");
    let panic_macro = ["unreachable", "!"].concat();

    assert!(!source.contains(&panic_macro));
    assert!(source.contains("command adapter returned a non-command binding"));
}

#[test]
fn raw_project_scene_snapshot_stays_inside_the_editor_crate() {
    let source = include_str!("snapshot.rs");

    assert!(
        source.contains("pub(crate) fn project_scene_snapshot("),
        "raw Scene snapshots must stay inside the editor crate"
    );
    assert!(
        !source.contains("pub fn project_scene_snapshot("),
        "EditorHostEventController must not expose raw Scene snapshots publicly"
    );
}
