use crate::ui::layouts::views::assets_activity_pane_data;
use slint::Model;
use zircon_runtime::ui::layout::UiSize;

const ASSETS_ACTIVITY_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/assets_activity.ui.toml"
));

#[test]
fn assets_activity_bootstrap_layout_self_hosts_shell_sections() {
    let layout = crate::tests::support::load_test_ui_asset(ASSETS_ACTIVITY_LAYOUT_TOML)
        .expect("assets activity layout");

    for required_node in [
        "assets_activity_root",
        "toolbar_panel",
        "toolbar_title_row",
        "toolbar_search_row",
        "toolbar_kind_primary_row",
        "toolbar_kind_secondary_row",
        "main_panel",
        "tree_panel",
        "content_panel",
        "utility_panel",
        "utility_tabs_row",
        "utility_content_panel",
        "preview_panel",
        "reference_left_panel",
        "reference_right_panel",
    ] {
        assert!(
            layout.contains_node(required_node),
            "assets activity bootstrap layout should include `{required_node}`"
        );
    }
}

#[test]
fn assets_activity_projection_maps_bootstrap_asset_into_mount_nodes() {
    let pane = assets_activity_pane_data(UiSize::new(1280.0, 820.0));
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();

    assert!(
        !nodes.is_empty(),
        "assets activity projection should produce template mount nodes"
    );

    let toolbar = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityToolbarPanel")
        .expect("toolbar panel node");
    assert_eq!(toolbar.role.to_string(), "Mount");
    assert!(toolbar.frame.width > 0.0 && toolbar.frame.height > 0.0);

    let tree = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityTreePanel")
        .expect("tree panel node");
    let content = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityContentPanel")
        .expect("content panel node");
    let utility = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityUtilityPanel")
        .expect("utility panel node");
    let references_right = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityReferenceRightPanel")
        .expect("right references node");

    assert!(tree.frame.x >= toolbar.frame.x);
    assert!(content.frame.x >= tree.frame.x + tree.frame.width);
    assert!(utility.frame.y >= tree.frame.y + tree.frame.height);
    assert!(references_right.frame.width > 0.0 && references_right.frame.height > 0.0);
}
