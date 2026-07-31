use zircon_runtime_interface::ui::layout::UiSize;

use super::{
    asset_content_paint_metadata, parse_activity_content_identity, AssetContentLayoutMetrics,
    AssetContentPaintMetadata, AssetContentPaintNodeInput, AssetContentRect, AssetContentSurface,
    AssetContentSurfaceProfile,
};
use super::{compact_file_like_display_name, RuntimeFileNameCompaction};
use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::retained_host::measure_runtime_text_width;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::snapshot::AssetViewMode;

#[test]
fn activity_content_starts_at_the_panel_origin_without_a_browser_header_gap() {
    let metrics = AssetContentLayoutMetrics::for_surface(
        AssetContentSurfaceProfile::Activity,
        AssetViewMode::List,
    );
    let viewport = metrics.viewport_frame(UiSize::new(226.0, 166.0));

    assert_eq!(viewport.y, 0.0);
    assert_eq!(viewport.height, 166.0);
    assert_eq!(metrics.first_row_y(), 8.0);
    assert_eq!(metrics.item_height, 38.0);
    assert_eq!(metrics.row_width(226.0), 210.0);
}

#[test]
fn browser_list_geometry_matches_the_painted_table_header_and_rows() {
    let metrics = AssetContentLayoutMetrics::for_surface(
        AssetContentSurfaceProfile::Browser,
        AssetViewMode::List,
    );
    let viewport = metrics.viewport_frame(UiSize::new(420.0, 220.0));

    assert_eq!(viewport.y, 24.0);
    assert_eq!(viewport.height, 196.0);
    assert_eq!(metrics.first_row_y(), 24.0);
    assert_eq!(metrics.folder_height, 28.0);
    assert_eq!(metrics.item_height, 28.0);
    assert_eq!(metrics.row_gap, 0.0);
}

#[test]
fn thumbnail_geometry_is_derived_from_the_same_dense_token_profile() {
    let metrics = AssetContentLayoutMetrics::for_surface(
        AssetContentSurfaceProfile::Activity,
        AssetViewMode::Thumbnail,
    );

    assert_eq!(metrics.folder_height, 60.0);
    assert_eq!(metrics.item_height, 88.0);
    assert_eq!(metrics.list_height(1, 2), 268.0);
}

#[test]
fn shared_content_name_compaction_preserves_the_file_extension_at_runtime_width() {
    let font_size = 13.333_333;
    let max_width = 112.0;
    let compact = compact_file_like_display_name(
        "workbench_extension_accessibility_workspace.zui",
        "zui",
        RuntimeFileNameCompaction {
            max_width,
            font_size,
            min_prefix_chars: 4,
            min_tail_stem_chars: 3,
            preferred_tail_stem_chars: 6,
        },
    );

    assert!(compact.contains("..."));
    assert!(compact.ends_with(".zui"));
    assert!(measure_runtime_text_width(&compact, font_size) <= max_width + 0.01);
}

#[test]
fn shared_content_name_compaction_does_not_linearly_shape_every_prefix() {
    let source = include_str!("text.rs");
    assert!(!source.contains("for prefix_count in"));
    assert!(source.contains("largest_fitting_candidate"));
}

#[test]
fn activity_generation_metadata_selects_fixed_nodes_and_only_visible_scroll_groups() {
    let nodes = view_model_with_asset_metadata(
        vec![
            node("AssetsActivityContentPanel", 0.0, 0.0, 100.0, 40.0),
            node("AssetsActivityContentFolderRow00", 0.0, 8.0, 100.0, 10.0),
            node("AssetsActivityContentFolderName00", 4.0, 9.0, 80.0, 8.0),
            node("AssetsActivityContentItemRow00", 0.0, 60.0, 100.0, 10.0),
            node("AssetsActivityContentItemName00", 4.0, 61.0, 80.0, 8.0),
            node("AssetsActivityPreviewPanel", 0.0, 90.0, 100.0, 20.0),
        ],
        AssetContentSurface::Activity,
    );
    let metadata = nodes
        .metadata::<AssetContentPaintMetadata>()
        .expect("generation metadata");

    assert_eq!(metadata.folder_row_count(), 1);
    assert_eq!(
        metadata.visible_node_rows(
            50.0,
            10.0,
            20.0,
            AssetContentRect {
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 40.0,
            },
        ),
        vec![0, 3, 4, 5]
    );
    assert!(parse_activity_content_identity("AssetsActivityContentItemName00").is_some());
}

#[test]
fn ten_thousand_browser_nodes_project_only_the_visible_thumbnail_groups() {
    let mut nodes = Vec::with_capacity(10_001);
    let mut grid = node("AssetBrowserThumbGridPanel", 0.0, 0.0, 100.0, 40.0);
    grid.value_number = 200_000.0;
    nodes.push(grid);
    for index in 0..10_000 {
        nodes.push(node(
            format!("AssetBrowserThumbCard{:05}", index + 1).as_str(),
            0.0,
            index as f32 * 20.0,
            100.0,
            10.0,
        ));
    }
    let nodes = view_model_with_asset_metadata(nodes, AssetContentSurface::Browser);
    let metadata = nodes
        .metadata::<AssetContentPaintMetadata>()
        .expect("generation metadata");

    let visible = metadata.visible_node_rows(
        100_000.0,
        0.0,
        0.0,
        AssetContentRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 40.0,
        },
    );

    assert_eq!(visible, vec![0, 5001, 5002]);
}

fn node(control_id: &str, x: f32, y: f32, width: f32, height: f32) -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        control_id: control_id.into(),
        frame: ViewTemplateFrameData {
            x,
            y,
            width,
            height,
        },
        ..ViewTemplateNodeData::default()
    }
}

fn view_model_with_asset_metadata(
    nodes: Vec<ViewTemplateNodeData>,
    surface: AssetContentSurface,
) -> ModelRc<ViewTemplateNodeData> {
    let metadata = asset_content_paint_metadata(
        nodes.iter().map(|node| {
            AssetContentPaintNodeInput::new(
                node.control_id.as_str(),
                node.frame.x,
                node.frame.y,
                node.frame.width,
                node.frame.height,
                node.value_number,
            )
        }),
        surface,
    );
    ModelRc::with_metadata(nodes, metadata)
}
