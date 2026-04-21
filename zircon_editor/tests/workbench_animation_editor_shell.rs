use std::fs;
use std::path::PathBuf;

fn read(relative: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|_| panic!("{relative} should be readable"))
}

#[test]
fn workbench_shell_declares_animation_editor_pane_surface_instead_of_fallback_only() {
    let pane_surface = read("ui/workbench/pane_surface.slint");
    let pane_content = read("ui/workbench/pane_content.slint");
    let pane_data = read("ui/workbench/pane_data.slint");
    let animation_pane = read("ui/workbench/animation_editor_pane.slint");

    assert!(pane_surface.contains("import { PaneContent } from \"pane_content.slint\";"));
    assert!(!pane_surface
        .contains("import { AnimationEditorPane } from \"animation_editor_pane.slint\";"));
    assert!(pane_data
        .contains("import { AnimationEditorPaneData } from \"animation_editor_pane.slint\";"));
    assert!(pane_data.contains("animation: AnimationEditorPaneData,"));
    assert!(pane_content.contains(
        "if !root.pane.show_empty && (root.pane.kind == \"AnimationSequenceEditor\" || root.pane.kind == \"AnimationGraphEditor\"): AnimationEditorPane {"
    ));
    assert!(pane_content.contains("pane: root.pane.animation;"));
    assert!(!pane_content.contains(
        "root.pane.kind == \"AnimationSequenceEditor\" || root.pane.kind == \"AnimationGraphEditor\" || root.pane.kind == \"UiAssetEditor\")"
    ));

    assert!(animation_pane.contains("export struct AnimationEditorPaneData {"));
    assert!(animation_pane.contains("export component AnimationEditorPane inherits Rectangle {"));
}
