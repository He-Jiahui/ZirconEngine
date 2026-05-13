use crate::ui::layouts::views::hierarchy_pane_nodes;
use crate::ui::workbench::snapshot::SceneEntry;
use zircon_runtime::ui::v2::UiV2AssetLoader;
use zircon_runtime_interface::ui::layout::UiSize;

const HIERARCHY_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/hierarchy.v2.ui.toml"
));

#[test]
fn hierarchy_bootstrap_layout_self_hosts_shell_sections() {
    let layout = UiV2AssetLoader::load_toml_str(HIERARCHY_LAYOUT_TOML).expect("hierarchy layout");

    for required_node in ["hierarchy_root", "list_panel"] {
        assert!(
            layout.nodes.contains_key(required_node),
            "hierarchy bootstrap layout should include `{required_node}`"
        );
    }
}

#[test]
fn hierarchy_projection_maps_bootstrap_asset_into_mount_nodes() {
    let pane = hierarchy_pane_nodes(
        &[SceneEntry {
            id: zircon_runtime::scene::NodeId::default(),
            name: "Root".to_string(),
            depth: 0,
            selected: true,
        }],
        UiSize::new(320.0, 640.0),
    );
    let nodes = (0..pane.row_count())
        .filter_map(|row| pane.row_data(row))
        .collect::<Vec<_>>();

    let list_panel = nodes
        .iter()
        .find(|node| node.control_id == "HierarchyListPanel")
        .expect("hierarchy list panel node");
    assert_eq!(list_panel.role.to_string(), "Mount");
    assert_eq!(list_panel.text.to_string(), "Root selected");
    assert!(list_panel.selected);
    assert!(list_panel.focused);
    assert_eq!(list_panel.surface_variant.to_string(), "panel");
    assert_eq!(list_panel.text_tone.to_string(), "default");
    assert!(
        list_panel.frame.width > 0.0 && list_panel.frame.height > 0.0,
        "expected hierarchy list panel frame to be laid out by the bootstrap asset"
    );
    assert!(list_panel.frame.x >= 0.0);
    assert!(list_panel.frame.y >= 0.0);
}
