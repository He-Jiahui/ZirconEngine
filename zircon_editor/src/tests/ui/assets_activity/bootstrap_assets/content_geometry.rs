use crate::ui::layouts::views::assets_activity_pane_data;
use crate::ui::workbench::asset_content_layout::{
    AssetContentLayoutMetrics, AssetContentSurfaceProfile,
};
use crate::ui::workbench::snapshot::{AssetItemSnapshot, AssetViewMode, AssetWorkspaceSnapshot};
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::layout::UiSize;

#[test]
fn assets_activity_content_rows_share_the_activity_pointer_geometry() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::List,
        selected_asset_uuid: Some("asset-selected".to_string()),
        visible_assets: vec![
            activity_asset(
                "asset-first",
                "editor_base.zui",
                ResourceKind::UiStyle,
                false,
            ),
            activity_asset(
                "asset-selected",
                "workbench_page_chrome.zui",
                ResourceKind::UiLayout,
                true,
            ),
        ]
        .into(),
        ..AssetWorkspaceSnapshot::default()
    };
    let pane = assets_activity_pane_data(&snapshot, UiSize::new(226.0, 346.0));
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let node = |control_id: &str| {
        nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .unwrap_or_else(|| panic!("missing Assets activity content node `{control_id}`"))
    };
    let content = node("AssetsActivityContentPanel");
    let first = node("AssetsActivityContentItemRow00");
    let selected = node("AssetsActivityContentItemRow01");
    let selected_name = node("AssetsActivityContentItemName01");
    let metrics = AssetContentLayoutMetrics::for_surface(
        AssetContentSurfaceProfile::Activity,
        AssetViewMode::List,
    );

    assert_eq!(first.frame.x, content.frame.x + metrics.row_x);
    assert_eq!(first.frame.y, content.frame.y + metrics.first_row_y());
    assert_eq!(first.frame.width, metrics.row_width(content.frame.width));
    assert_eq!(first.frame.height, metrics.item_height);
    assert_eq!(
        selected.frame.y,
        first.frame.y + metrics.item_height + metrics.row_gap
    );
    assert!(selected.selected);
    assert!(!selected_name.selected);
    assert_eq!(selected_name.border_width, 0.0);
    assert!(selected_name.text.to_string().ends_with(".zui"));
    assert!(selected_name.text.to_string().contains("..."));
    assert!(selected_name.frame.x >= selected.frame.x);
    assert!(
        selected_name.frame.x + selected_name.frame.width
            <= selected.frame.x + selected.frame.width
    );
    assert!(!nodes
        .iter()
        .any(|node| node.control_id == "AssetsActivityContentEmptyText"));
}

#[test]
fn empty_assets_activity_content_has_an_explicit_readable_state() {
    let pane = assets_activity_pane_data(
        &AssetWorkspaceSnapshot::default(),
        UiSize::new(226.0, 346.0),
    );
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let content = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityContentPanel")
        .expect("content panel");
    let empty = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityContentEmptyText")
        .expect("explicit empty content state");

    assert_eq!(empty.text.to_string(), "No assets in this folder");
    assert_eq!(empty.text_tone.to_string(), "muted");
    assert!(empty.frame.width > 0.0);
    assert!(empty.frame.x + empty.frame.width <= content.frame.x + content.frame.width);
}

#[test]
fn short_assets_activity_drawer_preserves_one_complete_asset_row_before_preview() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::List,
        visible_assets: vec![activity_asset(
            "asset-visible",
            "workbench_page_chrome.zui",
            ResourceKind::UiLayout,
            true,
        )]
        .into(),
        ..AssetWorkspaceSnapshot::default()
    };
    let pane = assets_activity_pane_data(&snapshot, UiSize::new(226.0, 224.0));
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let frame = |control_id: &str| {
        nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .map(|node| node.frame.clone())
            .unwrap_or_else(|| panic!("missing short drawer node `{control_id}`"))
    };
    let content = frame("AssetsActivityContentPanel");
    let row = frame("AssetsActivityContentItemRow00");
    let utility = frame("AssetsActivityUtilityPanel");
    let preview = frame("AssetsActivityPreviewPanel");

    assert!(row.width > 0.0 && row.height > 0.0);
    assert!(row.y + row.height <= content.y + content.height + f32::EPSILON);
    assert!(content.y + content.height <= utility.y + f32::EPSILON);
    assert!(preview.width > 0.0 && preview.height > 0.0);
}

#[test]
fn short_assets_activity_keeps_below_viewport_rows_in_scroll_source_geometry() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::List,
        visible_assets: (0..5)
            .map(|index| {
                activity_asset(
                    &format!("asset-{index}"),
                    &format!("workbench_asset_{index}.zui"),
                    ResourceKind::UiLayout,
                    false,
                )
            })
            .collect(),
        ..AssetWorkspaceSnapshot::default()
    };
    let pane = assets_activity_pane_data(&snapshot, UiSize::new(226.0, 224.0));
    let nodes = (0..pane.nodes.row_count())
        .filter_map(|row| pane.nodes.row_data(row))
        .collect::<Vec<_>>();
    let frame = |control_id: &str| {
        nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .map(|node| node.frame.clone())
            .unwrap_or_else(|| panic!("missing short scroll-source node `{control_id}`"))
    };
    let content_node = nodes
        .iter()
        .find(|node| node.control_id == "AssetsActivityContentPanel")
        .expect("short content panel");
    let content = content_node.frame.clone();
    let late_row = frame("AssetsActivityContentItemRow04");
    let metrics = AssetContentLayoutMetrics::for_surface(
        AssetContentSurfaceProfile::Activity,
        AssetViewMode::List,
    );

    assert!(late_row.width > 0.0 && late_row.height > 0.0);
    assert!(late_row.y >= content.y + content.height);
    assert_eq!(content_node.value_number, metrics.list_height(0, 5));
}

fn activity_asset(
    uuid: &str,
    display_name: &str,
    kind: ResourceKind,
    selected: bool,
) -> AssetItemSnapshot {
    AssetItemSnapshot {
        uuid: uuid.to_string(),
        locator: format!("res://ui/{display_name}"),
        display_name: display_name.to_string(),
        file_name: display_name.to_string(),
        extension: display_name
            .rsplit_once('.')
            .map(|(_, ext)| ext)
            .unwrap_or_default()
            .to_string(),
        kind,
        asset_type: crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
            kind,
        ),
        preview_artifact_path: String::new(),
        dirty: false,
        diagnostics: Vec::new(),
        selected,
        resource_state: None,
        resource_revision: Some(1),
    }
}
