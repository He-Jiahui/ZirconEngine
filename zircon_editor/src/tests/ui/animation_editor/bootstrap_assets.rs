use crate::ui::layouts::views::animation_editor_pane_nodes;
use slint::Model;
use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::layout::UiSize;

const ANIMATION_EDITOR_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/animation_editor.ui.toml"
));

#[test]
fn animation_editor_bootstrap_layout_self_hosts_shell_sections() {
    let layout = crate::tests::support::load_test_ui_asset(ANIMATION_EDITOR_LAYOUT_TOML)
        .expect("animation editor layout");

    for required_node in [
        "animation_editor_root",
        "header_panel",
        "header_mode_row",
        "header_path_row",
        "header_status_row",
        "body_panel",
        "sequence_content_panel",
        "sequence_timeline_row",
        "sequence_selection_row",
        "sequence_tracks_panel",
        "graph_content_panel",
        "graph_parameters_panel",
        "graph_nodes_panel",
        "state_machine_content_panel",
        "state_machine_entry_row",
        "state_machine_states_panel",
        "state_machine_transitions_panel",
    ] {
        assert!(
            layout.contains_node(required_node),
            "animation editor bootstrap layout should include `{required_node}`"
        );
    }
}

#[test]
fn animation_editor_projection_maps_bootstrap_asset_into_mount_nodes() {
    let pane = animation_editor_pane_nodes(UiSize::new(1280.0, 820.0));
    let nodes = (0..pane.row_count())
        .filter_map(|row| pane.row_data(row))
        .collect::<Vec<_>>();

    for label in [
        "AnimationEditorHeaderPanel",
        "AnimationEditorHeaderModeRow",
        "AnimationEditorHeaderPathRow",
        "AnimationEditorHeaderStatusRow",
        "AnimationEditorBodyPanel",
        "AnimationSequenceContentPanel",
        "AnimationSequenceTimelineRow",
        "AnimationSequenceSelectionRow",
        "AnimationSequenceTracksPanel",
        "AnimationGraphContentPanel",
        "AnimationGraphParametersPanel",
        "AnimationGraphNodesPanel",
        "AnimationStateMachineContentPanel",
        "AnimationStateMachineEntryRow",
        "AnimationStateMachineStatesPanel",
        "AnimationStateMachineTransitionsPanel",
    ] {
        let frame = nodes
            .iter()
            .find(|node| node.control_id == label)
            .expect("animation editor mount node")
            .frame
            .clone();
        assert!(
            frame.width > 0.0 && frame.height > 0.0,
            "expected `{label}` frame to be laid out by the bootstrap asset"
        );
    }

    let header = nodes
        .iter()
        .find(|node| node.control_id == "AnimationEditorHeaderPanel")
        .expect("header panel node");
    assert_eq!(header.role.to_string(), "Mount");

    let header_mode = nodes
        .iter()
        .find(|node| node.control_id == "AnimationEditorHeaderModeRow")
        .expect("header mode row node");
    let header_path = nodes
        .iter()
        .find(|node| node.control_id == "AnimationEditorHeaderPathRow")
        .expect("header path row node");
    let header_status = nodes
        .iter()
        .find(|node| node.control_id == "AnimationEditorHeaderStatusRow")
        .expect("header status row node");
    let body = nodes
        .iter()
        .find(|node| node.control_id == "AnimationEditorBodyPanel")
        .expect("body panel node");
    let sequence = nodes
        .iter()
        .find(|node| node.control_id == "AnimationSequenceContentPanel")
        .expect("sequence content node");
    let sequence_timeline = nodes
        .iter()
        .find(|node| node.control_id == "AnimationSequenceTimelineRow")
        .expect("sequence timeline node");
    let sequence_selection = nodes
        .iter()
        .find(|node| node.control_id == "AnimationSequenceSelectionRow")
        .expect("sequence selection node");
    let sequence_tracks = nodes
        .iter()
        .find(|node| node.control_id == "AnimationSequenceTracksPanel")
        .expect("sequence tracks node");
    let graph = nodes
        .iter()
        .find(|node| node.control_id == "AnimationGraphContentPanel")
        .expect("graph content node");
    let graph_parameters = nodes
        .iter()
        .find(|node| node.control_id == "AnimationGraphParametersPanel")
        .expect("graph parameters node");
    let graph_nodes = nodes
        .iter()
        .find(|node| node.control_id == "AnimationGraphNodesPanel")
        .expect("graph nodes node");
    let state_machine = nodes
        .iter()
        .find(|node| node.control_id == "AnimationStateMachineContentPanel")
        .expect("state machine content node");
    let state_machine_entry = nodes
        .iter()
        .find(|node| node.control_id == "AnimationStateMachineEntryRow")
        .expect("state machine entry node");
    let state_machine_states = nodes
        .iter()
        .find(|node| node.control_id == "AnimationStateMachineStatesPanel")
        .expect("state machine states node");
    let state_machine_transitions = nodes
        .iter()
        .find(|node| node.control_id == "AnimationStateMachineTransitionsPanel")
        .expect("state machine transitions node");

    assert!(header_mode.frame.y >= header.frame.y);
    assert!(header_path.frame.y >= header_mode.frame.y + header_mode.frame.height);
    assert!(header_status.frame.y >= header_path.frame.y + header_path.frame.height);
    assert!(body.frame.y >= header.frame.y + header.frame.height);
    assert_eq!(sequence.frame.x, graph.frame.x);
    assert_eq!(sequence.frame.x, state_machine.frame.x);
    assert_eq!(sequence.frame.y, graph.frame.y);
    assert_eq!(sequence.frame.y, state_machine.frame.y);
    assert!(sequence_timeline.frame.y >= sequence.frame.y);
    assert!(
        sequence_selection.frame.y >= sequence_timeline.frame.y + sequence_timeline.frame.height
    );
    assert!(
        sequence_tracks.frame.y >= sequence_selection.frame.y + sequence_selection.frame.height
    );
    assert!(graph_nodes.frame.y >= graph_parameters.frame.y + graph_parameters.frame.height);
    assert!(
        state_machine_states.frame.y
            >= state_machine_entry.frame.y + state_machine_entry.frame.height
    );
    assert!(
        state_machine_transitions.frame.y
            >= state_machine_states.frame.y + state_machine_states.frame.height
    );
}
