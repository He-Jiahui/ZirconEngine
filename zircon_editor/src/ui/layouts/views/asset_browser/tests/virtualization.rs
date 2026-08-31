use super::super::table_nodes::asset_table_row_index;
use super::super::thumbnail_nodes::{thumbnail_node_identity, ThumbnailNodeKind};
use super::*;

#[test]
fn ten_thousand_list_assets_keep_retained_rows_bounded_but_preserve_logical_extent() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::List,
        visible_assets: (1..=10_000).map(|index| asset_item(index, false)).collect(),
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(900.0, 620.0));
    let materialized_rows = nodes
        .iter()
        .filter(|node| asset_table_row_index(node.control_id.as_str()).is_some())
        .count();
    let table = find_node(&nodes, "AssetBrowserAssetTablePanel");

    assert!(
        materialized_rows <= 64,
        "retained list rows must follow viewport capacity, not logical item count: {materialized_rows}"
    );
    assert_eq!(
        table.value_number,
        10_000.0 * BROWSER_CONTENT_LIST_ROW_HEIGHT,
        "the scrollbar extent must continue to represent every logical asset"
    );
}

#[test]
fn ten_thousand_thumbnail_assets_keep_retained_cards_bounded_but_preserve_logical_extent() {
    let snapshot = AssetWorkspaceSnapshot {
        view_mode: AssetViewMode::Thumbnail,
        visible_assets: (1..=10_000).map(|index| asset_item(index, false)).collect(),
        ..AssetWorkspaceSnapshot::default()
    };

    let nodes = asset_browser_pane_nodes(&snapshot, UiSize::new(900.0, 620.0));
    let materialized_cards = nodes
        .iter()
        .filter(|node| {
            thumbnail_node_identity(node.control_id.as_str())
                .is_some_and(|(kind, _)| kind == ThumbnailNodeKind::Card)
        })
        .count();
    let grid = find_node(&nodes, BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID);
    let metrics = AssetThumbnailGridMetrics::new(grid.frame.width, 10_000);
    let logical_extent = metrics.content_extent();
    let expected_materialized_cards =
        metrics.materialized_item_budget(grid.frame.height, ASSET_BROWSER_VIRTUAL_OVERSCAN_ROWS);

    assert!(
        materialized_cards <= 64,
        "retained thumbnail cards must follow viewport capacity, not logical item count: {materialized_cards}"
    );
    assert_eq!(
        materialized_cards, expected_materialized_cards,
        "the retained pool must use the final grid column count instead of the six-column upper bound"
    );
    assert_eq!(
        grid.value_number, logical_extent,
        "the scrollbar extent must continue to represent every logical asset"
    );
}
