use std::fs;
use std::path::Path;

fn source(relative: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

fn sources(relatives: &[&str]) -> String {
    relatives
        .iter()
        .map(|relative| source(relative))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn animation_editor_shell_uses_canonical_sequence_and_graph_templates() {
    let panes = sources(&[
        "src/ui/retained_host/host_contract/data/panes.rs",
        "src/ui/retained_host/host_contract/data/panes/pane.rs",
        "src/ui/retained_host/host_contract/data/panes/animation.rs",
    ]);
    let sequence_asset = source("assets/ui/editor/host/animation_sequence_body.zui");
    let graph_asset = source("assets/ui/editor/host/animation_graph_body.zui");

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
        assert!(
            panes.contains(required),
            "animation pane DTO missing `{required}`"
        );
    }
    for required in [
        "AnimationEditorHeaderPanel",
        "AnimationSequencePaneBodyRoot",
        "animation_timeline_slot",
    ] {
        assert!(
            sequence_asset.contains(required),
            "animation sequence host template missing `{required}`"
        );
    }
    for required in [
        "AnimationEditorHeaderPanel",
        "AnimationGraphPaneBodyRoot",
        "animation_graph_canvas_slot",
    ] {
        assert!(
            graph_asset.contains(required),
            "animation graph host template missing `{required}`"
        );
    }
}
