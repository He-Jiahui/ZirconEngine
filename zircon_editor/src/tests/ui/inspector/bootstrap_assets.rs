use crate::ui::layouts::views::inspector_pane_nodes;
use crate::ui::workbench::snapshot::InspectorSnapshot;
use zircon_runtime::ui::v2::UiV2AssetLoader;
use zircon_runtime_interface::ui::layout::UiSize;

const INSPECTOR_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/inspector.v2.ui.toml"
));

#[test]
fn inspector_bootstrap_layout_self_hosts_shell_sections() {
    let layout = UiV2AssetLoader::load_toml_str(INSPECTOR_LAYOUT_TOML).expect("inspector layout");

    for required_node in [
        "inspector_root",
        "content_panel",
        "header_panel",
        "name_row",
        "parent_row",
        "position_row",
        "separator_row",
        "actions_row",
    ] {
        assert!(
            layout.nodes.contains_key(required_node),
            "inspector bootstrap layout should include `{required_node}`"
        );
    }
}

#[test]
fn inspector_projection_maps_bootstrap_asset_into_mount_nodes() {
    let pane = inspector_pane_nodes(
        Some(&InspectorSnapshot {
            id: zircon_runtime::scene::NodeId::default(),
            name: "Camera".to_string(),
            parent: "Root".to_string(),
            translation: ["1.0".to_string(), "2.0".to_string(), "3.0".to_string()],
            plugin_components: Vec::new(),
        }),
        UiSize::new(360.0, 520.0),
    );
    let nodes = (0..pane.row_count())
        .filter_map(|row| pane.row_data(row))
        .collect::<Vec<_>>();

    for label in [
        "InspectorContentPanel",
        "InspectorHeaderPanel",
        "InspectorNameRow",
        "InspectorParentRow",
        "InspectorPositionRow",
        "InspectorSeparatorRow",
        "InspectorActionsRow",
    ] {
        let frame = nodes
            .iter()
            .find(|node| node.control_id == label)
            .expect("inspector mount node")
            .frame
            .clone();
        assert!(
            frame.width > 0.0 && frame.height > 0.0,
            "expected `{label}` frame to be laid out by the bootstrap asset"
        );
    }

    let content = nodes
        .iter()
        .find(|node| node.control_id == "InspectorContentPanel")
        .expect("content panel");
    let header = nodes
        .iter()
        .find(|node| node.control_id == "InspectorHeaderPanel")
        .expect("header panel");
    assert_eq!(header.text.to_string(), "Inspector • Camera");
    let name = nodes
        .iter()
        .find(|node| node.control_id == "InspectorNameRow")
        .expect("name row");
    assert_eq!(name.text.to_string(), "Name • Camera");
    let parent = nodes
        .iter()
        .find(|node| node.control_id == "InspectorParentRow")
        .expect("parent row");
    assert_eq!(parent.text.to_string(), "Parent • Root");
    let position = nodes
        .iter()
        .find(|node| node.control_id == "InspectorPositionRow")
        .expect("position row");
    assert_eq!(position.text.to_string(), "Position • 1.0, 2.0, 3.0");
    let separator = nodes
        .iter()
        .find(|node| node.control_id == "InspectorSeparatorRow")
        .expect("separator row");
    let actions = nodes
        .iter()
        .find(|node| node.control_id == "InspectorActionsRow")
        .expect("actions row");
    assert!(header.selected);
    assert_eq!(header.text_tone.to_string(), "default");
    assert!(actions.selected);
    assert_eq!(actions.surface_variant.to_string(), "panel");

    assert!(header.frame.y >= content.frame.y);
    assert!(name.frame.y >= header.frame.y + header.frame.height);
    assert!(parent.frame.y >= name.frame.y + name.frame.height);
    assert!(position.frame.y >= parent.frame.y + parent.frame.height);
    assert!(separator.frame.y >= position.frame.y + position.frame.height);
    assert!(actions.frame.y >= separator.frame.y + separator.frame.height);
}
