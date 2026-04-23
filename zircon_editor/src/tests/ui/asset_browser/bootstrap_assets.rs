use crate::ui::layouts::views::asset_browser_pane_nodes;
use slint::Model;
use zircon_runtime::ui::layout::UiSize;

const ASSET_BROWSER_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/asset_browser.ui.toml"
));

#[test]
fn asset_browser_bootstrap_layout_self_hosts_shell_sections() {
    let layout =
        crate::tests::support::load_test_ui_asset(ASSET_BROWSER_LAYOUT_TOML).expect("asset layout");

    for required_node in [
        "asset_browser_root",
        "toolbar_panel",
        "toolbar_title_row",
        "toolbar_search_row",
        "toolbar_kind_primary_row",
        "toolbar_kind_secondary_row",
        "import_panel",
        "main_panel",
        "sources_panel",
        "content_panel",
        "details_panel",
        "utility_panel",
        "utility_tabs_row",
        "utility_content_panel",
        "reference_left_panel",
        "reference_right_panel",
    ] {
        assert!(
            layout.contains_node(required_node),
            "asset browser bootstrap layout should include `{required_node}`"
        );
    }
}

#[test]
fn asset_browser_projection_maps_bootstrap_asset_into_mount_nodes() {
    let pane = asset_browser_pane_nodes(UiSize::new(1280.0, 820.0));
    let nodes = (0..pane.row_count())
        .filter_map(|row| pane.row_data(row))
        .collect::<Vec<_>>();

    assert!(
        !nodes.is_empty(),
        "asset browser projection should produce template mount nodes"
    );

    let toolbar = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserToolbarPanel")
        .expect("toolbar panel node");
    assert_eq!(toolbar.role.to_string(), "Mount");
    assert!(toolbar.frame.width > 0.0 && toolbar.frame.height > 0.0);

    let import_panel = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserImportPanel")
        .expect("import panel node");
    let main = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserMainPanel")
        .expect("main panel node");
    let sources = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserSourcesPanel")
        .expect("sources panel node");
    let content = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserContentPanel")
        .expect("content panel node");
    let details = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserDetailsPanel")
        .expect("details panel node");
    let utility = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserUtilityPanel")
        .expect("utility panel node");
    let utility_tabs = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserUtilityTabsRow")
        .expect("utility tabs node");
    let utility_content = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserUtilityContentPanel")
        .expect("utility content node");
    let reference_left = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserReferenceLeftPanel")
        .expect("left references node");
    let reference_right = nodes
        .iter()
        .find(|node| node.control_id == "AssetBrowserReferenceRightPanel")
        .expect("right references node");

    assert!(import_panel.frame.y >= toolbar.frame.y + toolbar.frame.height);
    assert!(main.frame.y >= import_panel.frame.y + import_panel.frame.height);
    assert!(sources.frame.x >= main.frame.x);
    assert!(content.frame.x >= sources.frame.x + sources.frame.width);
    assert!(details.frame.x >= content.frame.x + content.frame.width);
    assert!(utility.frame.y >= main.frame.y + main.frame.height);
    assert!(utility_tabs.frame.y >= utility.frame.y);
    assert!(utility_content.frame.y >= utility_tabs.frame.y + utility_tabs.frame.height);
    assert!(reference_left.frame.width > 0.0 && reference_left.frame.height > 0.0);
    assert!(reference_right.frame.x >= reference_left.frame.x + reference_left.frame.width);
}
