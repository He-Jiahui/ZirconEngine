use super::super::table_nodes::asset_table_row_index;
use super::super::thumbnail_nodes::{thumbnail_node_identity, ThumbnailNodeKind};
use super::*;
use crate::ui::workbench::asset_content_layout::AssetContentPaintMetadata;
use std::hint::black_box;
use std::time::Instant;

const LARGE_CATALOG_ITEM_COUNT: usize = 100_000;
const LARGE_CATALOG_VIEWPORT_WIDTH: f32 = 900.0;
const LARGE_CATALOG_VIEWPORT_HEIGHT: f32 = 620.0;
const LARGE_CATALOG_MAX_MATERIALIZED_ITEMS: usize = 64;

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

#[test]
fn hundred_thousand_asset_pane_projection_preserves_extent_pool_and_selection() {
    let size = large_catalog_viewport();
    for view_mode in [AssetViewMode::List, AssetViewMode::Thumbnail] {
        let base = large_catalog_snapshot(view_mode);
        for selected_item in [None, Some(1), Some(50_000), Some(LARGE_CATALOG_ITEM_COUNT)] {
            let snapshot = with_selected_item(&base, selected_item);
            let nodes = asset_browser_pane_data(&snapshot, size).nodes;
            let metadata = nodes
                .metadata::<AssetContentPaintMetadata>()
                .expect("asset browser paint metadata");
            assert_eq!(
                metadata.browser_logical_item_count(),
                LARGE_CATALOG_ITEM_COUNT
            );
            assert!(metadata.browser_has_virtual_items());
            assert!(
                metadata.browser_materialized_item_count() <= LARGE_CATALOG_MAX_MATERIALIZED_ITEMS
            );
            let selected_index = selected_item.map(|index| index - 1);
            let first_binding = metadata
                .browser_slot_binding(0.0, 0)
                .expect("the first logical asset must bind at the initial scroll position");
            assert_eq!(first_binding.logical_index, 0);
            assert_eq!(first_binding.selected, selected_index == Some(0));
            let first_node = nodes
                .iter()
                .find(|node| match view_mode {
                    AssetViewMode::List => {
                        asset_table_row_index(node.control_id.as_str()) == Some(0)
                    }
                    AssetViewMode::Thumbnail => thumbnail_node_identity(node.control_id.as_str())
                        .is_some_and(|(kind, index)| kind == ThumbnailNodeKind::Card && index == 0),
                })
                .expect("the first asset must have a materialized row or card");
            assert_eq!(first_node.selected, selected_index == Some(0));
            let selection_locator = find_node(&nodes, "AssetBrowserSelectionLocatorText");
            let expected_locator = selected_item
                .map(|index| format!("res://asset-{index:02}"))
                .unwrap_or_else(|| "Select an asset to inspect".to_string());
            assert_eq!(selection_locator.text.as_str(), expected_locator.as_str());

            match view_mode {
                AssetViewMode::List => {
                    let row_count = nodes
                        .iter()
                        .filter(|node| asset_table_row_index(node.control_id.as_str()).is_some())
                        .count();
                    let expected_count = asset_browser_materialized_item_budget(
                        view_mode,
                        size.height,
                        LARGE_CATALOG_ITEM_COUNT,
                        ASSET_BROWSER_VIRTUAL_OVERSCAN_ROWS,
                    );
                    assert_eq!(row_count, expected_count);
                    assert!(row_count > 0);
                    assert!(row_count <= LARGE_CATALOG_MAX_MATERIALIZED_ITEMS);
                    assert_eq!(
                        find_node(&nodes, "AssetBrowserAssetTablePanel").value_number,
                        LARGE_CATALOG_ITEM_COUNT as f32 * BROWSER_CONTENT_LIST_ROW_HEIGHT,
                    );
                }
                AssetViewMode::Thumbnail => {
                    let card_count = nodes
                        .iter()
                        .filter(|node| {
                            thumbnail_node_identity(node.control_id.as_str())
                                .is_some_and(|(kind, _)| kind == ThumbnailNodeKind::Card)
                        })
                        .count();
                    let grid = find_node(&nodes, BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID);
                    let metrics =
                        AssetThumbnailGridMetrics::new(grid.frame.width, LARGE_CATALOG_ITEM_COUNT);
                    assert_eq!(
                        card_count,
                        metrics.materialized_item_budget(
                            grid.frame.height,
                            ASSET_BROWSER_VIRTUAL_OVERSCAN_ROWS,
                        )
                    );
                    assert!(card_count > 0);
                    assert!(card_count <= LARGE_CATALOG_MAX_MATERIALIZED_ITEMS);
                    assert_eq!(grid.value_number, metrics.content_extent());
                    assert!(grid.value_number > grid.frame.height);
                }
            }

            if let Some(selected_index) = selected_index.filter(|index| *index > 0) {
                let scroll_px = match view_mode {
                    AssetViewMode::List => selected_index as f32 * BROWSER_CONTENT_LIST_ROW_HEIGHT,
                    AssetViewMode::Thumbnail => {
                        let grid = find_node(&nodes, BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID);
                        AssetThumbnailGridMetrics::new(grid.frame.width, LARGE_CATALOG_ITEM_COUNT)
                            .item_frame(selected_index)
                            .expect("selected thumbnail must have a logical frame")
                            .y
                    }
                };
                let selected_binding = (0..metadata.browser_materialized_item_count())
                    .filter_map(|slot| metadata.browser_slot_binding(scroll_px, slot))
                    .find(|binding| binding.logical_index == selected_index)
                    .expect("selected middle or tail asset must bind at its scroll position");
                assert_eq!(selected_binding.logical_index, selected_index);
                assert!(selected_binding.selected);
                assert!(
                    (0..metadata.browser_materialized_item_count())
                        .filter_map(|slot| metadata.browser_slot_binding(scroll_px, slot))
                        .all(|binding| binding.logical_index == selected_index || !binding.selected),
                    "neighboring virtual slots must remain unselected"
                );
            }
        }
    }
}

#[test]
#[ignore = "run the 100k Asset Browser pane projection baseline in Windows Release"]
fn editor57_hundred_thousand_asset_pane_projection_release_benchmark() {
    const MARKER: &str = "EDITOR57_100K_ASSET_PANE_PROJECTION_BENCH_V1";
    const WARMUPS: usize = 5;
    const SAMPLES: usize = 31;
    let size = large_catalog_viewport();

    for view_mode in [AssetViewMode::List, AssetViewMode::Thumbnail] {
        let view_label = match view_mode {
            AssetViewMode::List => "list",
            AssetViewMode::Thumbnail => "thumbnail",
        };
        let base = large_catalog_snapshot(view_mode);
        for (phase, prime_prior, prior_selection, target_selection) in [
            ("initial", false, None, None),
            ("select_first", true, None, Some(1)),
            ("select_middle", true, Some(1), Some(50_000)),
            (
                "select_last",
                true,
                Some(50_000),
                Some(LARGE_CATALOG_ITEM_COUNT),
            ),
        ] {
            let prior = prime_prior.then(|| with_selected_item(&base, prior_selection));
            let target = with_selected_item(&base, target_selection);
            for _ in 0..WARMUPS {
                black_box(measure_large_catalog_projection(
                    &target,
                    prior.as_ref(),
                    size,
                ));
            }
            let mut samples_ns = Vec::with_capacity(SAMPLES);
            for _ in 0..SAMPLES {
                samples_ns.push(measure_large_catalog_projection(
                    &target,
                    prior.as_ref(),
                    size,
                ));
            }
            let (p50_ns, p95_ns, p99_ns) = large_catalog_percentiles(&samples_ns);
            println!(
                "PERF_RESULT {MARKER} mode={view_label} phase={phase} items={LARGE_CATALOG_ITEM_COUNT} viewport={}x{} warmups={WARMUPS} samples={SAMPLES} p50_ns={p50_ns} p95_ns={p95_ns} p99_ns={p99_ns} raw_samples_ns={samples_ns:?} os={} arch={} package_version={}",
                size.width,
                size.height,
                std::env::consts::OS,
                std::env::consts::ARCH,
                env!("CARGO_PKG_VERSION"),
            );
            assert!(p50_ns > 0 && p95_ns >= p50_ns && p99_ns >= p95_ns);
        }
    }
}

fn large_catalog_viewport() -> UiSize {
    UiSize::new(LARGE_CATALOG_VIEWPORT_WIDTH, LARGE_CATALOG_VIEWPORT_HEIGHT)
}

fn large_catalog_snapshot(view_mode: AssetViewMode) -> AssetWorkspaceSnapshot {
    AssetWorkspaceSnapshot {
        catalog_revision: 1,
        view_mode,
        visible_assets: (1..=LARGE_CATALOG_ITEM_COUNT)
            .map(|index| asset_item(index, false))
            .collect(),
        ..AssetWorkspaceSnapshot::default()
    }
}

fn with_selected_item(
    base: &AssetWorkspaceSnapshot,
    selected_item: Option<usize>,
) -> AssetWorkspaceSnapshot {
    let mut snapshot = base.clone();
    snapshot.selected_asset_uuid = selected_item.map(|index| format!("asset-{index:02}"));
    snapshot
}

fn measure_large_catalog_projection(
    target: &AssetWorkspaceSnapshot,
    prior: Option<&AssetWorkspaceSnapshot>,
    size: UiSize,
) -> u128 {
    if let Some(prior) = prior {
        black_box(asset_browser_pane_data(prior, size));
    } else {
        clear_asset_browser_pane_projection_cache_for_tests();
    }
    let started = Instant::now();
    let data = asset_browser_pane_data(target, size);
    let elapsed_ns = started.elapsed().as_nanos();
    black_box(data.nodes.row_count());
    elapsed_ns
}

fn large_catalog_percentiles(samples: &[u128]) -> (u128, u128, u128) {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let percentile = |percent: usize| sorted[(sorted.len() * percent).div_ceil(100) - 1];
    (percentile(50), percentile(95), percentile(99))
}
