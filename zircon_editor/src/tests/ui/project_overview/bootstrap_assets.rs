use crate::ui::layouts::views::project_overview_pane_data;
use crate::ui::workbench::snapshot::ProjectOverviewSnapshot;
use slint::Model;
use zircon_runtime::ui::layout::UiSize;

const PROJECT_OVERVIEW_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/project_overview.ui.toml"
));

#[test]
fn project_overview_bootstrap_layout_self_hosts_shell_sections() {
    let layout = crate::tests::support::load_test_ui_asset(PROJECT_OVERVIEW_LAYOUT_TOML)
        .expect("project overview layout");

    for required_node in [
        "project_overview_root",
        "outer_panel",
        "header_title_row",
        "header_path_row",
        "details_panel",
        "catalog_panel",
    ] {
        assert!(
            layout.contains_node(required_node),
            "project overview bootstrap layout should include `{required_node}`"
        );
    }
}

#[test]
fn project_overview_projection_maps_bootstrap_asset_into_template_nodes() {
    let pane = project_overview_pane_data(
        &ProjectOverviewSnapshot {
            project_name: "Sample Project".to_string(),
            project_root: "E:/Projects/SampleProject".to_string(),
            assets_root: "res://".to_string(),
            library_root: "E:/Projects/SampleProject/Library".to_string(),
            default_scene_uri: "res://scenes/main.scene".to_string(),
            catalog_revision: 42,
            folder_count: 7,
            asset_count: 21,
        },
        UiSize::new(960.0, 640.0),
    );
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();

    assert!(
        !nodes.is_empty(),
        "project overview projection should produce visible template nodes"
    );

    let title = nodes
        .iter()
        .find(|node| node.control_id == "ProjectOverviewTitleText")
        .expect("title node");
    assert_eq!(title.text.to_string(), "Sample Project");

    let summary = nodes
        .iter()
        .find(|node| node.control_id == "ProjectOverviewCatalogSummaryValue")
        .expect("catalog summary node");
    assert_eq!(summary.text.to_string(), "7 folders • 21 assets");

    let open_assets = nodes
        .iter()
        .find(|node| node.control_id == "OpenAssetsView")
        .expect("open assets button");
    assert_eq!(open_assets.role.to_string(), "Button");
    assert_eq!(open_assets.dispatch_kind.to_string(), "surface");
    assert_eq!(open_assets.action_id.to_string(), "OpenView.editor.assets");

    let open_browser = nodes
        .iter()
        .find(|node| node.control_id == "OpenAssetBrowser")
        .expect("asset browser button");
    assert_eq!(open_browser.role.to_string(), "Button");
    assert_eq!(open_browser.dispatch_kind.to_string(), "asset");
}
