use zircon_runtime::ui::v2::UiV2AssetLoader;

const ANIMATION_GRAPH_BODY_ZUI: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/host/animation_graph_body.zui"
));
const ANIMATION_SEQUENCE_BODY_ZUI: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/host/animation_sequence_body.zui"
));

#[test]
fn animation_editor_host_templates_expose_native_mount_slots() {
    for (name, source, slot_node, slot_name) in [
        (
            "animation graph",
            ANIMATION_GRAPH_BODY_ZUI,
            "canvas_slot",
            "animation_graph_canvas_slot",
        ),
        (
            "animation sequence",
            ANIMATION_SEQUENCE_BODY_ZUI,
            "timeline_slot",
            "animation_timeline_slot",
        ),
    ] {
        let document = UiV2AssetLoader::load_toml_str(source)
            .unwrap_or_else(|error| panic!("load {name} host template: {error}"));

        for required_node in ["root", "header", "mode", "path", "status", slot_node] {
            assert!(
                document.nodes.contains_key(required_node),
                "{name} host template should include `{required_node}`"
            );
        }
        assert!(
            source.contains(&format!("slot = {{ name = \"{slot_name}\" }}")),
            "{name} host template should mount `{slot_name}`"
        );
        assert!(
            source.contains(&format!("props = {{ slot_name = \"{slot_name}\" }}")),
            "{name} native slot should publish `{slot_name}`"
        );
    }
}

#[test]
fn animation_editor_host_templates_do_not_publish_placeholder_actions() {
    assert!(!ANIMATION_GRAPH_BODY_ZUI.contains("AnimationCommand.AddGraphNode"));
    assert!(!ANIMATION_SEQUENCE_BODY_ZUI.contains("AnimationCommand.ScrubTimeline"));
}
