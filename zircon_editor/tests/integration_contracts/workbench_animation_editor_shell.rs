use std::fs;
use std::path::Path;

fn source(relative: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn animation_editor_shell_uses_template_nodes_and_toml_panels() {
    let panes = source("src/ui/slint_host/host_contract/data/panes.rs");
    let asset = source("assets/ui/editor/animation_editor.ui.toml");

    for required in [
        "pub(crate) struct AnimationEditorPaneData",
        "pub nodes: ModelRc<TemplatePaneNodeData>",
        "pub track_items: ModelRc<SharedString>",
        "pub parameter_items: ModelRc<SharedString>",
        "pub node_items: ModelRc<SharedString>",
        "pub state_items: ModelRc<SharedString>",
        "pub transition_items: ModelRc<SharedString>",
        "pub animation: AnimationEditorPaneData",
    ] {
        assert!(panes.contains(required), "animation pane DTO missing `{required}`");
    }
    for required in [
        "AnimationEditorHeaderPanel",
        "AnimationSequenceTimelineRow",
        "AnimationGraphContentPanel",
        "AnimationStateMachineTransitionsPanel",
    ] {
        assert!(asset.contains(required), "animation editor TOML missing `{required}`");
    }
}
