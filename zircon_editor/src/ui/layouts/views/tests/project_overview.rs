use super::*;
use zircon_runtime_interface::ui::design_tokens::EditorDensityTokens;

fn snapshot() -> ProjectOverviewSnapshot {
    ProjectOverviewSnapshot {
        project_name: "Sample Project".to_string(),
        project_root: "E:/Projects/SampleProject".to_string(),
        assets_root: "res://assets".to_string(),
        cache_root: "E:/Projects/SampleProject/.zircon/cache".to_string(),
        default_scene_uri: "res://scenes/main.scene".to_string(),
        catalog_revision: 42,
        folder_count: 7,
        asset_count: 21,
    }
}

fn node_by_control_id<'a>(
    nodes: &'a [crate::ui::layouts::views::ViewTemplateNodeData],
    control_id: &str,
) -> Option<&'a crate::ui::layouts::views::ViewTemplateNodeData> {
    nodes.iter().find(|node| node.control_id == control_id)
}

#[test]
fn project_actions_use_the_shared_workbench_control_height() {
    let pane = project_overview_pane_data(&snapshot(), UiSize::new(360.0, 520.0));
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();

    assert!(nodes.iter().any(|node| node.control_id == "OpenAssetsView"));
    assert!(nodes
        .iter()
        .any(|node| node.control_id == "OpenAssetBrowser"));

    let Some(open_assets) = node_by_control_id(&nodes, "OpenAssetsView") else {
        return;
    };
    let Some(open_browser) = node_by_control_id(&nodes, "OpenAssetBrowser") else {
        return;
    };

    assert_eq!(
        open_assets.frame.height,
        EditorDensityTokens::WORKBENCH_ROW_HEIGHT
    );
    assert_eq!(
        open_browser.frame.height,
        EditorDensityTokens::WORKBENCH_ROW_HEIGHT
    );
    assert!(open_assets.frame.width > 200.0);
    assert!(open_browser.frame.y >= open_assets.frame.y + open_assets.frame.height);
}

#[test]
fn project_settings_keep_value_fields_reachable_in_a_narrow_drawer() {
    let pane = project_overview_pane_data(&snapshot(), UiSize::new(160.0, 520.0));
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();

    for control_id in [
        "ProjectOverviewDefaultSceneValue",
        "ProjectOverviewAssetsRootValue",
        "ProjectOverviewLibraryValue",
    ] {
        let value = node_by_control_id(&nodes, control_id)
            .unwrap_or_else(|| panic!("missing project settings value `{control_id}`"));
        assert!(
            value.frame.width > 0.0,
            "{control_id} must retain visible value space in a narrow drawer: {:?}",
            value.frame
        );
        assert!(
            value.frame.x + value.frame.width <= 160.0,
            "{control_id} must stay within the narrow drawer: {:?}",
            value.frame
        );
    }
}
