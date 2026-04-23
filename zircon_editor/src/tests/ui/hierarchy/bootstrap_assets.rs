use crate::ui::layouts::views::hierarchy_pane_nodes;
use slint::Model;
use zircon_runtime::ui::layout::UiSize;

const HIERARCHY_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/hierarchy.ui.toml"
));

#[test]
fn hierarchy_bootstrap_layout_self_hosts_shell_sections() {
    let layout =
        crate::tests::support::load_test_ui_asset(HIERARCHY_LAYOUT_TOML).expect("hierarchy layout");

    for required_node in ["hierarchy_root", "list_panel"] {
        assert!(
            layout.contains_node(required_node),
            "hierarchy bootstrap layout should include `{required_node}`"
        );
    }
}

#[test]
fn hierarchy_projection_maps_bootstrap_asset_into_mount_nodes() {
    let pane = hierarchy_pane_nodes(UiSize::new(320.0, 640.0));
    let nodes = (0..pane.row_count())
        .filter_map(|row| pane.row_data(row))
        .collect::<Vec<_>>();

    let list_panel = nodes
        .iter()
        .find(|node| node.control_id == "HierarchyListPanel")
        .expect("hierarchy list panel node");
    assert_eq!(list_panel.role.to_string(), "Mount");
    assert!(
        list_panel.frame.width > 0.0 && list_panel.frame.height > 0.0,
        "expected hierarchy list panel frame to be laid out by the bootstrap asset"
    );
    assert!(list_panel.frame.x >= 0.0);
    assert!(list_panel.frame.y >= 0.0);
}
