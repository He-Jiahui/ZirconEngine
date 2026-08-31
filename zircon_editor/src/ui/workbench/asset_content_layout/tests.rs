use zircon_runtime_interface::ui::layout::UiSize;

use super::{
    asset_content_paint_metadata, parse_activity_content_identity, ActivityAssetReferenceListKind,
    ActivityContentNodeIdentity, ActivityContentNodeRole, AssetContentLayoutMetrics,
    AssetContentPaintMetadata, AssetContentPaintNodeInput, AssetContentRect,
    AssetContentRowDescriptor, AssetContentSurface, AssetContentSurfaceProfile,
    BrowserThumbnailNodeRole,
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
    assert!(matches!(
        metadata.row_descriptor(1),
        AssetContentRowDescriptor::ActivityContent(ActivityContentNodeIdentity::Folder {
            index: 0,
            role: ActivityContentNodeRole::Row,
        })
    ));
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
fn activity_tree_metadata_publishes_count_and_row_addresses_without_removing_fixed_paint_rows() {
    let nodes = view_model_with_asset_metadata(
        vec![
            node(
                "Workspace/AssetsActivityTreeRowPanel",
                4.0,
                10.0,
                92.0,
                18.0,
            ),
            node("AssetsActivityTreeLabel", 12.0, 14.0, 70.0, 10.0),
            node("AssetsActivityTreeRowPanel", 4.0, 34.0, 92.0, 18.0),
        ],
        AssetContentSurface::Activity,
    );
    let metadata = nodes
        .metadata::<AssetContentPaintMetadata>()
        .expect("generation metadata");

    assert_eq!(metadata.asset_tree_row_count(), 2);
    assert_eq!(metadata.activity_tree_node_row(0), Some(0));
    assert_eq!(metadata.activity_tree_node_row(1), Some(2));
    assert_eq!(
        metadata.row_descriptor(0),
        AssetContentRowDescriptor::ActivityTreeRow
    );
    assert_eq!(
        metadata.visible_activity_node_rows(
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            AssetContentRect {
                x: 0.0,
                y: 0.0,
                width: 120.0,
                height: 80.0,
            },
        ),
        vec![0, 1, 2],
        "tree rows remain fixed paint rows until Activity tree virtualization owns them"
    );
}

#[test]
fn browser_tree_metadata_counts_published_logical_row_groups() {
    let metadata = asset_content_paint_metadata(
        [
            AssetContentPaintNodeInput::new(
                "AssetBrowserSourcesRowPanel",
                0.0,
                0.0,
                100.0,
                20.0,
                0.0,
            ),
            AssetContentPaintNodeInput::new(
                "AssetBrowserSourcesTreeRow2/AssetBrowserSourcesRowPanel",
                0.0,
                22.0,
                100.0,
                20.0,
                0.0,
            ),
            AssetContentPaintNodeInput::new(
                "AssetBrowserSourcesTreeRow2/AssetBrowserSourcesNameText",
                8.0,
                26.0,
                80.0,
                10.0,
                0.0,
            ),
        ]
        .into_iter(),
        AssetContentSurface::Browser,
    );

    assert_eq!(metadata.asset_tree_row_count(), 2);
    assert!(matches!(
        metadata.row_descriptor(1),
        AssetContentRowDescriptor::BrowserSourceTree { index: 1 }
    ));
}

#[test]
fn activity_reference_metadata_virtualizes_each_list_with_its_own_scroll_offset() {
    let nodes = view_model_with_asset_metadata(
        vec![
            node(
                "AssetsActivityReferenceLeftScrollBody",
                0.0,
                20.0,
                100.0,
                60.0,
            ),
            node(
                "AssetsActivityReferenceLeftRowPanel01",
                0.0,
                20.0,
                96.0,
                34.0,
            ),
            node(
                "AssetsActivityReferenceLeftRowNameText01",
                8.0,
                24.0,
                56.0,
                10.0,
            ),
            node(
                "AssetsActivityReferenceLeftRowPanel02",
                0.0,
                58.0,
                96.0,
                34.0,
            ),
            node(
                "AssetsActivityReferenceLeftRowNameText02",
                8.0,
                62.0,
                56.0,
                10.0,
            ),
            node(
                "AssetsActivityReferenceRightScrollBody",
                120.0,
                20.0,
                100.0,
                60.0,
            ),
            node(
                "AssetsActivityReferenceRightRowPanel01",
                120.0,
                20.0,
                96.0,
                34.0,
            ),
            node(
                "AssetsActivityReferenceRightRowPanel02",
                120.0,
                58.0,
                96.0,
                34.0,
            ),
        ],
        AssetContentSurface::Activity,
    );
    let metadata = nodes
        .metadata::<AssetContentPaintMetadata>()
        .expect("generation metadata");

    assert_eq!(
        metadata.visible_activity_node_rows(
            0.0,
            40.0,
            0.0,
            0.0,
            0.0,
            AssetContentRect {
                x: 0.0,
                y: 0.0,
                width: 240.0,
                height: 120.0,
            },
        ),
        vec![0, 3, 4, 5, 6, 7],
        "the scrolled References list must not keep its first row in the painter plan"
    );
    assert_eq!(
        metadata.row_descriptor(1),
        AssetContentRowDescriptor::ActivityReference {
            list_kind: ActivityAssetReferenceListKind::References,
            index: 0,
            paints_hover: true,
        }
    );
    assert_eq!(
        metadata.row_descriptor(2),
        AssetContentRowDescriptor::ActivityReference {
            list_kind: ActivityAssetReferenceListKind::References,
            index: 0,
            paints_hover: false,
        }
    );
}

#[test]
fn browser_generation_descriptor_preserves_thumbnail_roles_without_paint_time_strings() {
    let inputs = [
        "AssetBrowserThumbGridPanel",
        "AssetBrowserThumbCard01",
        "AssetBrowserThumbSelectionMarker01",
        "AssetBrowserThumbVisual01",
        "AssetBrowserThumbInfoBand01",
        "AssetBrowserThumbNameContinuation01",
        "AssetBrowserThumbName01",
        "AssetBrowserThumbTypeBadge01",
        "AssetBrowserThumbType01",
        "AssetBrowserThumbMeta01",
    ];
    let metadata = asset_content_paint_metadata(
        inputs.iter().enumerate().map(|(index, control_id)| {
            AssetContentPaintNodeInput::new(control_id, 0.0, index as f32 * 16.0, 100.0, 12.0, 0.0)
        }),
        AssetContentSurface::Browser,
    );

    for (row, role) in [
        (1, BrowserThumbnailNodeRole::Card),
        (2, BrowserThumbnailNodeRole::SelectionMarker),
        (3, BrowserThumbnailNodeRole::Visual),
        (4, BrowserThumbnailNodeRole::InfoBand),
        (5, BrowserThumbnailNodeRole::NameContinuation),
        (6, BrowserThumbnailNodeRole::Name),
        (7, BrowserThumbnailNodeRole::TypeBadge),
        (8, BrowserThumbnailNodeRole::Type),
        (9, BrowserThumbnailNodeRole::Meta),
    ] {
        assert!(matches!(
            metadata.row_descriptor(row),
            AssetContentRowDescriptor::BrowserContent(
                super::BrowserContentNodeIdentity::Thumbnail { index: 0, role: actual }
            ) if actual == role
        ));
    }
}

#[test]
fn generation_metadata_reports_one_identity_parse_per_input_row() {
    let metadata = asset_content_paint_metadata(
        [
            AssetContentPaintNodeInput::new("AssetBrowserTable", 0.0, 0.0, 100.0, 24.0, 0.0),
            AssetContentPaintNodeInput::new("AssetBrowserItem01", 0.0, 24.0, 100.0, 24.0, 0.0),
            AssetContentPaintNodeInput::new("AssetBrowserItem02", 0.0, 48.0, 100.0, 24.0, 0.0),
        ]
        .into_iter(),
        AssetContentSurface::Browser,
    );

    assert_eq!(metadata.identity_parse_count(), 3);
}

#[test]
fn browser_generation_descriptors_own_content_and_auxiliary_viewport_geometry() {
    let metadata = asset_content_paint_metadata(
        [
            AssetContentPaintNodeInput::new(
                "Workspace/AssetBrowserTablePanel",
                10.0,
                12.0,
                240.0,
                180.0,
                640.0,
            ),
            AssetContentPaintNodeInput::new(
                "Workspace/AssetBrowserTableHeader",
                10.0,
                12.0,
                240.0,
                24.0,
                0.0,
            ),
            AssetContentPaintNodeInput::new(
                "Workspace/AssetBrowserContentPreview",
                10.0,
                160.0,
                240.0,
                32.0,
                0.0,
            ),
            AssetContentPaintNodeInput::new(
                "AssetBrowserSourcesScrollBody",
                2.0,
                40.0,
                120.0,
                96.0,
                0.0,
            ),
            AssetContentPaintNodeInput::new(
                "AssetBrowserReferenceLeftScrollBody",
                260.0,
                40.0,
                120.0,
                48.0,
                0.0,
            ),
            AssetContentPaintNodeInput::new(
                "AssetBrowserReferenceRightScrollBody",
                260.0,
                96.0,
                120.0,
                48.0,
                0.0,
            ),
        ]
        .into_iter(),
        AssetContentSurface::Browser,
    );

    assert_eq!(
        metadata.content_panel(),
        Some(AssetContentRect {
            x: 10.0,
            y: 12.0,
            width: 240.0,
            height: 180.0,
        })
    );
    assert_eq!(
        metadata.viewport(),
        Some(AssetContentRect {
            x: 10.0,
            y: 36.0,
            width: 240.0,
            height: 124.0,
        })
    );
    assert_eq!(metadata.content_extent(), 640.0);
    assert_eq!(
        metadata.browser_source_tree_viewport(),
        Some(AssetContentRect {
            x: 2.0,
            y: 40.0,
            width: 120.0,
            height: 96.0,
        })
    );
    assert_eq!(
        metadata.browser_reference_viewport(super::BrowserAssetReferenceListKind::References),
        Some(AssetContentRect {
            x: 260.0,
            y: 40.0,
            width: 120.0,
            height: 48.0,
        })
    );
    assert_eq!(
        metadata.browser_reference_viewport(super::BrowserAssetReferenceListKind::UsedBy),
        Some(AssetContentRect {
            x: 260.0,
            y: 96.0,
            width: 120.0,
            height: 48.0,
        })
    );
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

    let (visible, visible_item_count) = metadata.visible_browser_node_rows(
        100_000.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        AssetContentRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 40.0,
        },
    );

    assert_eq!(metadata.browser_materialized_item_count(), 10_000);
    assert_eq!(metadata.browser_materialized_node_count(), 10_000);
    assert_eq!(visible_item_count, 2);
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
