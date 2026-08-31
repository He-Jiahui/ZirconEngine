use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostPaneInteractionStateData, TemplateNodeFrameData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::paint_template_nodes::TemplateNodePaintTransform;
use crate::ui::retained_host::measure_runtime_text_width;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::asset_content_layout::{
    asset_content_paint_metadata, parse_activity_content_identity, parse_browser_content_identity,
    ActivityContentNodeIdentity, ActivityContentNodeRole, AssetBrowserLogicalPaintGeneration,
    AssetBrowserPaintItem, AssetBrowserThumbnailPaintItem, AssetContentPaintNodeInput,
    AssetContentSurface, BrowserContentNodeIdentity,
};

use super::{ActivityAssetContentProjector, BrowserAssetContentProjector};

#[test]
fn projector_consumers_use_generation_metadata_without_model_scans_or_identity_parsing() {
    let production = include_str!("projector.rs");

    assert!(!production.contains("row_data("));
    assert!(!production.contains("for row in 0..nodes.row_count()"));
    assert!(!production.contains("parse_activity_content_identity("));
    assert!(!production.contains("parse_browser_content_identity("));
    assert!(production.contains("metadata_rc::<AssetContentPaintMetadata>"));
}

#[test]
fn activity_asset_content_identity_maps_rows_and_children_without_aliases() {
    assert_eq!(
        parse_activity_content_identity("AssetsActivityContentPanel"),
        Some(ActivityContentNodeIdentity::ContentPanel)
    );
    assert_eq!(
        parse_activity_content_identity("AssetsActivityContentFolderRow02"),
        Some(ActivityContentNodeIdentity::Folder {
            index: 2,
            role: ActivityContentNodeRole::Row,
        })
    );
    assert_eq!(
        parse_activity_content_identity("AssetsActivityContentFolderName02"),
        Some(ActivityContentNodeIdentity::Folder {
            index: 2,
            role: ActivityContentNodeRole::Name,
        })
    );
    assert_eq!(
        parse_activity_content_identity("mount/AssetsActivityContentItemMeta11"),
        Some(ActivityContentNodeIdentity::Item {
            index: 11,
            role: ActivityContentNodeRole::Meta,
        })
    );
    assert_eq!(
        parse_activity_content_identity("AssetsActivityPreviewPanel"),
        None
    );
    assert_eq!(
        parse_activity_content_identity("AssetsActivityContentItemRow"),
        None
    );
}

#[test]
fn activity_asset_content_projector_scrolls_clips_and_hovers_shared_row_index() {
    let nodes = activity_model(vec![
        node("AssetsActivityContentPanel", 10.0, 20.0, 100.0, 40.0),
        node("AssetsActivityContentFolderRow00", 12.0, 20.0, 96.0, 12.0),
        node("AssetsActivityContentItemRow00", 12.0, 60.0, 96.0, 12.0),
        node("AssetsActivityContentItemName00", 20.0, 62.0, 60.0, 8.0),
        node("AssetsActivityPreviewPanel", 10.0, 70.0, 100.0, 20.0),
    ]);
    let interaction = HostPaneInteractionStateData {
        activity_asset_content_scroll_px: 30.0,
        activity_asset_content_hovered_index: 1,
        ..HostPaneInteractionStateData::default()
    };
    let projector =
        ActivityAssetContentProjector::new(&nodes, &frame(5.0, 7.0, 120.0, 100.0), &interaction)
            .expect("content panel projector");
    let pane_clip = frame(0.0, 0.0, 200.0, 200.0);

    assert_eq!(
        projector
            .row_visit_indices(nodes.row_count(), &pane_clip)
            .expect("generation row plan"),
        vec![0, 2, 3, 4]
    );

    assert!(projector
        .transform_row(1, nodes.row_data(1).expect("folder row"), pane_clip.clone())
        .is_none());
    let (item, item_clip) = projector
        .transform_row(2, nodes.row_data(2).expect("item row"), pane_clip.clone())
        .expect("scrolled item row should enter content viewport");
    assert_eq!(item.frame.y, 30.0);
    assert!(item.hovered);
    assert_eq!(item_clip, frame(15.0, 27.0, 100.0, 40.0));

    let (name, name_clip) = projector
        .transform_row(3, nodes.row_data(3).expect("item name"), pane_clip.clone())
        .expect("item child should share projection and clip");
    assert_eq!(name.frame.y, 32.0);
    assert!(!name.hovered);
    assert_eq!(name_clip, item_clip);

    let (preview, preview_clip) = projector
        .transform_row(4, nodes.row_data(4).expect("preview"), pane_clip.clone())
        .expect("unrelated utility nodes pass through");
    assert_eq!(preview.frame.y, 70.0);
    assert_eq!(preview_clip, pane_clip);
}

#[test]
fn activity_asset_content_projector_falls_back_without_panel_and_ignores_stale_hover() {
    let no_panel = activity_model(vec![node(
        "AssetsActivityContentItemRow00",
        0.0,
        0.0,
        20.0,
        10.0,
    )]);
    assert!(ActivityAssetContentProjector::new(
        &no_panel,
        &frame(0.0, 0.0, 40.0, 40.0),
        &HostPaneInteractionStateData::default(),
    )
    .is_none());

    let nodes = activity_model(vec![
        node("AssetsActivityContentPanel", 0.0, 0.0, 40.0, 40.0),
        node("AssetsActivityContentItemRow00", 0.0, 0.0, 40.0, 10.0),
    ]);
    let interaction = HostPaneInteractionStateData {
        activity_asset_content_hovered_index: 99,
        ..HostPaneInteractionStateData::default()
    };
    let projector =
        ActivityAssetContentProjector::new(&nodes, &frame(0.0, 0.0, 40.0, 40.0), &interaction)
            .expect("projector");
    let (row, _) = projector
        .transform_row(
            1,
            nodes.row_data(1).expect("item row"),
            frame(0.0, 0.0, 40.0, 40.0),
        )
        .expect("visible row");
    assert!(!row.hovered);
}

#[test]
fn activity_asset_content_projector_keeps_empty_state_visible_with_stale_scroll() {
    let nodes = activity_model(vec![
        node("AssetsActivityContentPanel", 0.0, 0.0, 80.0, 40.0),
        node("AssetsActivityContentEmptyText", 4.0, 4.0, 72.0, 12.0),
    ]);
    let interaction = HostPaneInteractionStateData {
        activity_asset_content_scroll_px: 120.0,
        activity_asset_content_hovered_index: 3,
        ..HostPaneInteractionStateData::default()
    };
    let projector =
        ActivityAssetContentProjector::new(&nodes, &frame(10.0, 20.0, 80.0, 40.0), &interaction)
            .expect("empty content projector");

    let (empty, clip) = projector
        .transform_row(
            1,
            nodes.row_data(1).expect("empty node"),
            frame(0.0, 0.0, 120.0, 100.0),
        )
        .expect("empty state must remain visible");

    assert_eq!(empty.frame.y, 4.0);
    assert!(!empty.hovered);
    assert_eq!(clip, frame(10.0, 20.0, 80.0, 40.0));
}

#[test]
fn activity_reference_projector_virtualizes_and_projects_each_list_independently() {
    let nodes = activity_model(vec![
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
    ]);
    let interaction = HostPaneInteractionStateData {
        activity_asset_references_scroll_px: 40.0,
        activity_asset_references_hovered_index: 1,
        activity_asset_used_by_hovered_index: 0,
        ..HostPaneInteractionStateData::default()
    };
    let projector =
        ActivityAssetContentProjector::new(&nodes, &frame(5.0, 7.0, 240.0, 100.0), &interaction)
            .expect("reference viewport should not require an unrelated content panel");
    let pane_clip = frame(0.0, 0.0, 260.0, 120.0);

    assert_eq!(
        projector
            .row_visit_indices(nodes.row_count(), &pane_clip)
            .expect("reference visibility plan"),
        vec![0, 3, 4, 5, 6, 7],
    );
    assert!(
        projector
            .transform_row(
                1,
                nodes.row_data(1).expect("first references row"),
                pane_clip.clone(),
            )
            .is_none(),
        "the References scroll offset must cull the first row"
    );

    let (references_row, references_clip) = projector
        .transform_row(
            3,
            nodes.row_data(3).expect("second references row"),
            pane_clip.clone(),
        )
        .expect("the second References row enters after scroll");
    assert_eq!(references_row.frame.y, 18.0);
    assert!(references_row.hovered);
    assert_eq!(references_clip, frame(5.0, 27.0, 100.0, 60.0));

    let (used_by_row, _) = projector
        .transform_row(6, nodes.row_data(6).expect("first Used By row"), pane_clip)
        .expect("the unscrolled Used By list remains independent");
    assert_eq!(used_by_row.frame.y, 20.0);
    assert!(used_by_row.hovered);
}

#[test]
fn browser_content_identity_and_projector_keep_header_fixed_and_clip_rows() {
    assert_eq!(
        parse_browser_content_identity("AssetBrowserAssetTablePanel"),
        Some(BrowserContentNodeIdentity::TablePanel)
    );
    assert_eq!(
        parse_browser_content_identity("WorkbenchAssetBrowserAssetRow03"),
        Some(BrowserContentNodeIdentity::Row { index: 2 })
    );

    let nodes = browser_model(vec![
        node("AssetBrowserAssetTablePanel", 10.0, 20.0, 120.0, 80.0),
        node("WorkbenchAssetBrowserTableHeader", 10.0, 20.0, 120.0, 24.0),
        node("WorkbenchAssetBrowserAssetRow01", 10.0, 44.0, 120.0, 28.0),
        node("WorkbenchAssetBrowserAssetRow02", 10.0, 72.0, 120.0, 28.0),
        node("AssetBrowserContentPreviewCard", 10.0, 90.0, 120.0, 40.0),
    ]);
    let interaction = HostPaneInteractionStateData {
        browser_asset_content_scroll_px: 28.0,
        browser_asset_content_hovered_index: 1,
        ..HostPaneInteractionStateData::default()
    };
    let projector =
        BrowserAssetContentProjector::new(&nodes, &frame(5.0, 7.0, 150.0, 180.0), &interaction)
            .expect("browser table projector");
    let pane_clip = frame(0.0, 0.0, 200.0, 200.0);

    assert_eq!(
        projector
            .row_visit_indices(nodes.row_count(), &pane_clip)
            .expect("generation row plan"),
        vec![0, 1, 3, 4]
    );

    let (header, header_clip) = projector
        .transform_row(1, nodes.row_data(1).expect("header"), pane_clip.clone())
        .expect("header remains visible");
    assert_eq!(header.frame.y, 20.0);
    assert_eq!(header_clip, pane_clip);
    assert!(projector
        .transform_row(2, nodes.row_data(2).expect("first row"), pane_clip.clone())
        .is_none());
    let (second, row_clip) = projector
        .transform_row(3, nodes.row_data(3).expect("second row"), pane_clip.clone())
        .expect("second row scrolls into viewport");
    assert_eq!(second.frame.y, 44.0);
    assert!(second.hovered);
    assert_eq!(row_clip, frame(15.0, 51.0, 120.0, 46.0));

    let (preview, preview_clip) = projector
        .transform_row(4, nodes.row_data(4).expect("preview"), pane_clip.clone())
        .expect("preview remains outside scroll projection");
    assert_eq!(preview.frame.y, 90.0);
    assert_eq!(preview_clip, pane_clip);
}

#[test]
fn browser_source_tree_projector_scrolls_clips_and_hovers_dynamic_rows() {
    let nodes = browser_model(vec![
        node("AssetBrowserAssetTablePanel", 160.0, 20.0, 120.0, 80.0),
        node("WorkbenchAssetBrowserTableHeader", 160.0, 20.0, 120.0, 24.0),
        node("AssetBrowserSourcesScrollBody", 0.0, 49.0, 136.0, 60.0),
        node("AssetBrowserSourcesRowPanel", 8.0, 57.0, 120.0, 28.0),
        node(
            "AssetBrowserSourcesTreeRow02/AssetBrowserSourcesRowPanel",
            8.0,
            89.0,
            120.0,
            28.0,
        ),
    ]);
    let interaction = HostPaneInteractionStateData {
        browser_asset_tree_scroll_px: 40.0,
        browser_asset_tree_hovered_index: 1,
        ..HostPaneInteractionStateData::default()
    };
    let projector =
        BrowserAssetContentProjector::new(&nodes, &frame(5.0, 7.0, 300.0, 140.0), &interaction)
            .expect("browser projector");
    let pane_clip = frame(0.0, 0.0, 400.0, 200.0);

    assert_eq!(
        projector
            .row_visit_indices(nodes.row_count(), &pane_clip)
            .expect("browser source tree visibility plan"),
        vec![0, 1, 2, 4],
        "offscreen source rows must not be visited as fixed template nodes"
    );

    assert!(projector
        .transform_row(
            3,
            nodes.row_data(3).expect("first tree row"),
            pane_clip.clone()
        )
        .is_none());
    let (row, row_clip) = projector
        .transform_row(4, nodes.row_data(4).expect("second tree row"), pane_clip)
        .expect("second tree row enters the viewport after scrolling");
    assert_eq!(row.frame.y, 49.0);
    assert!(row.hovered);
    assert_eq!(row_clip, frame(5.0, 56.0, 136.0, 60.0));
}

#[test]
fn browser_reference_projector_survives_without_main_content_and_virtualizes_each_list() {
    let nodes = browser_model(vec![
        node(
            "AssetBrowserReferenceLeftScrollBody",
            0.0,
            20.0,
            100.0,
            60.0,
        ),
        node("AssetBrowserReferenceLeftRowPanel01", 0.0, 20.0, 96.0, 34.0),
        node(
            "AssetBrowserReferenceLeftRowNameText01",
            8.0,
            24.0,
            56.0,
            10.0,
        ),
        node("AssetBrowserReferenceLeftRowPanel02", 0.0, 58.0, 96.0, 34.0),
        node(
            "AssetBrowserReferenceLeftRowNameText02",
            8.0,
            62.0,
            56.0,
            10.0,
        ),
    ]);
    let interaction = HostPaneInteractionStateData {
        browser_asset_references_scroll_px: 40.0,
        browser_asset_references_hovered_index: 1,
        ..HostPaneInteractionStateData::default()
    };
    let projector =
        BrowserAssetContentProjector::new(&nodes, &frame(5.0, 7.0, 140.0, 100.0), &interaction)
            .expect("reference viewport should not require the unrelated content table");
    let pane_clip = frame(0.0, 0.0, 180.0, 120.0);

    assert_eq!(
        projector
            .row_visit_indices(nodes.row_count(), &pane_clip)
            .expect("reference visibility plan"),
        vec![0, 3, 4],
        "offscreen reference rows must not remain fixed painter nodes"
    );
    assert!(projector
        .transform_row(
            1,
            nodes.row_data(1).expect("first reference row"),
            pane_clip.clone(),
        )
        .is_none());

    let (row, clip) = projector
        .transform_row(
            3,
            nodes.row_data(3).expect("second reference row"),
            pane_clip.clone(),
        )
        .expect("second reference row enters after scroll");
    assert_eq!(row.frame.y, 18.0);
    assert!(row.hovered);
    assert_eq!(clip, frame(5.0, 27.0, 100.0, 60.0));

    let (label, _) = projector
        .transform_row(
            4,
            nodes.row_data(4).expect("second reference label"),
            pane_clip,
        )
        .expect("reference label follows its row");
    assert_eq!(label.frame.y, 22.0);
    assert!(!label.hovered);
}

#[test]
fn browser_reference_projector_uses_independent_scroll_and_hover_state_per_list() {
    let nodes = browser_model(vec![
        node(
            "AssetBrowserReferenceLeftScrollBody",
            0.0,
            20.0,
            100.0,
            60.0,
        ),
        node("AssetBrowserReferenceLeftRowPanel01", 0.0, 20.0, 96.0, 34.0),
        node("AssetBrowserReferenceLeftRowPanel02", 0.0, 58.0, 96.0, 34.0),
        node(
            "AssetBrowserReferenceRightScrollBody",
            120.0,
            20.0,
            100.0,
            60.0,
        ),
        node(
            "AssetBrowserReferenceRightRowPanel01",
            120.0,
            20.0,
            96.0,
            34.0,
        ),
        node(
            "AssetBrowserReferenceRightRowPanel02",
            120.0,
            58.0,
            96.0,
            34.0,
        ),
    ]);
    let interaction = HostPaneInteractionStateData {
        browser_asset_references_scroll_px: 40.0,
        browser_asset_references_hovered_index: 1,
        browser_asset_used_by_scroll_px: 0.0,
        browser_asset_used_by_hovered_index: 0,
        ..HostPaneInteractionStateData::default()
    };
    let projector =
        BrowserAssetContentProjector::new(&nodes, &frame(5.0, 7.0, 240.0, 100.0), &interaction)
            .expect("reference viewports should initialize independently");
    let pane_clip = frame(0.0, 0.0, 260.0, 120.0);

    assert!(
        projector
            .transform_row(
                1,
                nodes.row_data(1).expect("first references row"),
                pane_clip.clone(),
            )
            .is_none(),
        "the References list scroll must not leave its first row visible"
    );
    let (references_row, _) = projector
        .transform_row(
            2,
            nodes.row_data(2).expect("second references row"),
            pane_clip.clone(),
        )
        .expect("second references row after its independent scroll");
    assert_eq!(references_row.frame.y, 18.0);
    assert!(references_row.hovered);

    let (used_by_row, _) = projector
        .transform_row(4, nodes.row_data(4).expect("first used-by row"), pane_clip)
        .expect("Used By remains unscrolled");
    assert_eq!(used_by_row.frame.y, 20.0);
    assert!(used_by_row.hovered);
}

#[test]
fn browser_thumbnail_projector_scrolls_card_children_and_keeps_grid_fixed() {
    let nodes = browser_model(vec![
        node("AssetBrowserThumbGridPanel", 10.0, 20.0, 120.0, 80.0),
        node("AssetBrowserThumbCard01", 12.0, 28.0, 52.0, 48.0),
        node("AssetBrowserThumbInfoBand01", 16.0, 58.0, 44.0, 14.0),
        node("AssetBrowserThumbCard02", 68.0, 28.0, 52.0, 48.0),
        node("AssetBrowserThumbInfoBand02", 72.0, 58.0, 44.0, 14.0),
        node("AssetBrowserThumbName02", 76.0, 61.0, 36.0, 10.0),
    ]);
    let interaction = HostPaneInteractionStateData {
        browser_asset_content_scroll_px: 16.0,
        browser_asset_content_hovered_index: 1,
        ..HostPaneInteractionStateData::default()
    };
    let projector =
        BrowserAssetContentProjector::new(&nodes, &frame(5.0, 7.0, 150.0, 120.0), &interaction)
            .expect("browser thumbnail projector");
    let pane_clip = frame(0.0, 0.0, 200.0, 200.0);

    let (grid, grid_clip) = projector
        .transform_row(0, nodes.row_data(0).expect("grid"), pane_clip.clone())
        .expect("grid remains fixed");
    assert_eq!(grid.frame.y, 20.0);
    assert_eq!(grid_clip, pane_clip);

    let (card, card_clip) = projector
        .transform_row(
            3,
            nodes.row_data(3).expect("second card"),
            pane_clip.clone(),
        )
        .expect("second card remains visible after scroll");
    assert_eq!(card.frame.y, 12.0);
    assert!(card.hovered);
    assert_eq!(card_clip, frame(15.0, 27.0, 120.0, 80.0));

    let (band, _) = projector
        .transform_row(
            4,
            nodes.row_data(4).expect("second info band"),
            pane_clip.clone(),
        )
        .expect("second info band follows its card");
    assert_eq!(band.frame.y, 42.0);
    assert!(band.hovered);

    let (name, _) = projector
        .transform_row(5, nodes.row_data(5).expect("second name"), pane_clip)
        .expect("second name follows its card");
    assert_eq!(name.frame.y, 45.0);
    assert!(!name.hovered);
}

#[test]
fn browser_thumbnail_projector_rebinds_bounded_slots_after_a_deep_scroll() {
    let nodes = browser_virtual_model(
        vec![
            node("AssetBrowserThumbGridPanel", 0.0, 0.0, 120.0, 80.0),
            node("AssetBrowserThumbCard01", 0.0, 8.0, 52.0, 48.0),
            node("AssetBrowserThumbName01", 4.0, 12.0, 44.0, 10.0),
            node("AssetBrowserThumbCard02", 60.0, 8.0, 52.0, 48.0),
            node("AssetBrowserThumbName02", 64.0, 12.0, 44.0, 10.0),
            node("AssetBrowserThumbCard03", 0.0, 68.0, 52.0, 48.0),
            node("AssetBrowserThumbName03", 4.0, 72.0, 44.0, 10.0),
            node("AssetBrowserThumbCard04", 60.0, 68.0, 52.0, 48.0),
            node("AssetBrowserThumbName04", 64.0, 72.0, 44.0, 10.0),
        ],
        (0..200)
            .map(|index| {
                AssetBrowserPaintItem::Thumbnail(AssetBrowserThumbnailPaintItem {
                    name: format!("Logical asset {index}"),
                    source_file_name: String::new(),
                    file_extension: String::new(),
                    name_continuation: String::new(),
                    type_label: "Mesh".to_string(),
                    type_label_width: 20.0,
                    state_label: "Ready".to_string(),
                    visual_variant: "asset-mesh".to_string(),
                    preview_artifact_path: String::new(),
                })
            })
            .collect(),
        vec![4],
    );
    let interaction = HostPaneInteractionStateData {
        browser_asset_content_scroll_px: 120.0,
        browser_asset_content_hovered_index: 4,
        ..HostPaneInteractionStateData::default()
    };
    let projector =
        BrowserAssetContentProjector::new(&nodes, &frame(0.0, 0.0, 120.0, 80.0), &interaction)
            .expect("virtual browser projector");
    let pane_clip = frame(0.0, 0.0, 120.0, 80.0);

    let (card, _) = projector
        .transform_row(
            1,
            nodes.row_data(1).expect("recycled first retained card"),
            pane_clip.clone(),
        )
        .expect("entering logical item should bind to the recycled row");
    let (name, _) = projector
        .transform_row(
            2,
            nodes.row_data(2).expect("recycled first retained name"),
            pane_clip,
        )
        .expect("slot label should follow the logical binding");

    assert_eq!(card.frame.y, 8.0);
    assert!(card.hovered, "slot 2 should represent logical item 4");
    assert!(
        card.selected,
        "selection belongs to the logical item binding"
    );
    assert_eq!(name.text.as_str(), "Logical asset 4");
}

#[test]
fn browser_thumbnail_virtual_rebind_reprojects_item_specific_child_geometry() {
    let mut slots = vec![node("AssetBrowserThumbGridPanel", 0.0, 0.0, 300.0, 320.0)];
    for (index, x, y) in [
        (1, 8.0, 8.0),
        (2, 148.0, 8.0),
        (3, 8.0, 166.0),
        (4, 148.0, 166.0),
    ] {
        slots.extend(thumbnail_slot_nodes(index, x, y));
    }
    let mut items = (0..200)
        .map(|index| {
            AssetBrowserPaintItem::Thumbnail(AssetBrowserThumbnailPaintItem {
                name: format!("Asset {index}"),
                source_file_name: String::new(),
                file_extension: String::new(),
                name_continuation: String::new(),
                type_label: "Mesh".to_string(),
                type_label_width: 20.0,
                state_label: "Ready".to_string(),
                visual_variant: "asset-mesh".to_string(),
                preview_artifact_path: String::new(),
            })
        })
        .collect::<Vec<_>>();
    items[4] = AssetBrowserPaintItem::Thumbnail(AssetBrowserThumbnailPaintItem {
        name: "workbench_extension_accessibility_workspace.zui".to_string(),
        source_file_name: "workbench_extension_accessibility_workspace.zui".to_string(),
        file_extension: "zui".to_string(),
        name_continuation: "Accessibility workspace".to_string(),
        type_label: "MaterialInstance".to_string(),
        type_label_width: 160.0,
        state_label: "Ready".to_string(),
        visual_variant: "asset-material".to_string(),
        preview_artifact_path: String::new(),
    });
    let nodes = browser_virtual_model(slots, items, vec![4]);
    let interaction = HostPaneInteractionStateData {
        browser_asset_content_scroll_px: 166.0,
        browser_asset_content_hovered_index: 4,
        ..HostPaneInteractionStateData::default()
    };
    let projector =
        BrowserAssetContentProjector::new(&nodes, &frame(0.0, 0.0, 300.0, 320.0), &interaction)
            .expect("virtual browser projector");
    let pane_clip = frame(0.0, 0.0, 300.0, 320.0);

    let (card, _) = projector
        .transform_row(
            1,
            nodes.row_data(1).expect("recycled first retained card"),
            pane_clip.clone(),
        )
        .expect("slot should bind to logical item four");
    let (visual, _) = projector
        .transform_row(
            2,
            nodes.row_data(2).expect("recycled first retained visual"),
            pane_clip.clone(),
        )
        .expect("visual should reproject with the bound item");
    let (info_band, _) = projector
        .transform_row(
            3,
            nodes
                .row_data(3)
                .expect("recycled first retained info band"),
            pane_clip.clone(),
        )
        .expect("info band should reproject with the bound item");
    let (continuation, _) = projector
        .transform_row(
            6,
            nodes
                .row_data(6)
                .expect("recycled first retained continuation"),
            pane_clip.clone(),
        )
        .expect("continuation should gain a visible line for the bound item");
    let (name, _) = projector
        .transform_row(
            5,
            nodes.row_data(5).expect("recycled first retained name"),
            pane_clip.clone(),
        )
        .expect("name should fit the rebound slot width");
    let (badge, _) = projector
        .transform_row(
            7,
            nodes.row_data(7).expect("recycled first retained badge"),
            pane_clip.clone(),
        )
        .expect("type badge should use the bound item width");
    let (type_label, _) = projector
        .transform_row(
            8,
            nodes
                .row_data(8)
                .expect("recycled first retained type label"),
            pane_clip.clone(),
        )
        .expect("type label should fit the rebound badge");
    let (meta, _) = projector
        .transform_row(
            9,
            nodes.row_data(9).expect("recycled first retained meta"),
            pane_clip,
        )
        .expect("meta should follow the rebound badge");

    assert_eq!(card.frame.y, 158.0);
    assert!(card.hovered);
    assert!(
        visual.frame.height < 88.0,
        "stacked info band reduces visual height"
    );
    assert!(
        info_band.frame.y > visual.frame.y + visual.frame.height,
        "stacked item-specific info band follows the shorter preview"
    );
    assert!(continuation.frame.height > 0.0);
    assert!(continuation.frame.y > name.frame.y);
    assert!(
        badge.frame.width > 42.0,
        "wide type labels expand the badge"
    );
    assert_eq!(type_label.text.as_str(), "MaterialInstance");
    assert_eq!(type_label.frame.y, badge.frame.y);
    assert!(type_label.frame.x > badge.frame.x);
    assert!(
        type_label.frame.x + type_label.frame.width <= badge.frame.x + badge.frame.width,
        "type text must stay inside the rebound badge"
    );
    assert!(meta.frame.x > badge.frame.x + badge.frame.width);
    assert!(name.text.ends_with(".zui"));
    assert!(
        measure_runtime_text_width(name.text.as_str(), name.font_size) <= name.frame.width + 0.01
    );
}

fn thumbnail_slot_nodes(index: usize, x: f32, y: f32) -> Vec<ViewTemplateNodeData> {
    let mut nodes = [
        node(
            format!("AssetBrowserThumbCard{index:02}").as_str(),
            x,
            y,
            132.0,
            150.0,
        ),
        node(
            format!("AssetBrowserThumbVisual{index:02}").as_str(),
            x + 8.0,
            y + 8.0,
            116.0,
            88.0,
        ),
        node(
            format!("AssetBrowserThumbInfoBand{index:02}").as_str(),
            x + 8.0,
            y + 92.0,
            116.0,
            42.0,
        ),
        node(
            format!("AssetBrowserThumbName{index:02}").as_str(),
            x + 13.0,
            y + 97.0,
            106.0,
            16.0,
        ),
        node(
            format!("AssetBrowserThumbNameContinuation{index:02}").as_str(),
            x + 13.0,
            y + 113.0,
            106.0,
            0.0,
        ),
        node(
            format!("AssetBrowserThumbTypeBadge{index:02}").as_str(),
            x + 13.0,
            y + 117.0,
            42.0,
            13.0,
        ),
        node(
            format!("AssetBrowserThumbType{index:02}").as_str(),
            x + 18.0,
            y + 117.0,
            32.0,
            13.0,
        ),
        node(
            format!("AssetBrowserThumbMeta{index:02}").as_str(),
            x + 60.0,
            y + 117.0,
            59.0,
            13.0,
        ),
        node(
            format!("AssetBrowserThumbSelectionMarker{index:02}").as_str(),
            x + 8.0,
            y + 92.0,
            0.0,
            42.0,
        ),
    ]
    .to_vec();
    let selection_marker = nodes.remove(8);
    nodes.insert(3, selection_marker);
    for node in &mut nodes {
        node.font_size = 13.333_333;
    }
    nodes
}

fn activity_model(nodes: Vec<ViewTemplateNodeData>) -> ModelRc<TemplatePaneNodeData> {
    host_model(nodes, AssetContentSurface::Activity)
}

fn browser_model(nodes: Vec<ViewTemplateNodeData>) -> ModelRc<TemplatePaneNodeData> {
    host_model(nodes, AssetContentSurface::Browser)
}

fn browser_virtual_model(
    nodes: Vec<ViewTemplateNodeData>,
    items: Vec<AssetBrowserPaintItem>,
    selected_item_indices: Vec<usize>,
) -> ModelRc<TemplatePaneNodeData> {
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
        AssetContentSurface::Browser,
    )
    .with_browser_virtual_items(
        AssetBrowserLogicalPaintGeneration::from_chunks(vec![items.into()]),
        selected_item_indices,
        0,
    );
    ModelRc::with_metadata(nodes, metadata).map_preserving_metadata(|node| TemplatePaneNodeData {
        control_id: node.control_id.clone(),
        value_number: node.value_number,
        font_size: node.font_size,
        frame: TemplateNodeFrameData {
            x: node.frame.x,
            y: node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        },
        ..TemplatePaneNodeData::default()
    })
}

fn host_model(
    nodes: Vec<ViewTemplateNodeData>,
    surface: AssetContentSurface,
) -> ModelRc<TemplatePaneNodeData> {
    view_asset_content_model(nodes, surface).map_preserving_metadata(|node| TemplatePaneNodeData {
        control_id: node.control_id.clone(),
        value_number: node.value_number,
        frame: TemplateNodeFrameData {
            x: node.frame.x,
            y: node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        },
        ..TemplatePaneNodeData::default()
    })
}

fn view_asset_content_model(
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

fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
