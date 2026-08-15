use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostPaneInteractionStateData, TemplateNodeFrameData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::paint_template_nodes::TemplateNodePaintTransform;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::asset_content_layout::{
    asset_content_paint_metadata, parse_activity_content_identity, parse_browser_content_identity,
    ActivityContentNodeIdentity, ActivityContentNodeRole, AssetContentPaintNodeInput,
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
        .transform(nodes.row_data(1).expect("folder row"), pane_clip.clone())
        .is_none());
    let (item, item_clip) = projector
        .transform(nodes.row_data(2).expect("item row"), pane_clip.clone())
        .expect("scrolled item row should enter content viewport");
    assert_eq!(item.frame.y, 30.0);
    assert!(item.hovered);
    assert_eq!(item_clip, frame(15.0, 27.0, 100.0, 40.0));

    let (name, name_clip) = projector
        .transform(nodes.row_data(3).expect("item name"), pane_clip.clone())
        .expect("item child should share projection and clip");
    assert_eq!(name.frame.y, 32.0);
    assert!(!name.hovered);
    assert_eq!(name_clip, item_clip);

    let (preview, preview_clip) = projector
        .transform(nodes.row_data(4).expect("preview"), pane_clip.clone())
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
        .transform(
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
        .transform(
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
            .transform(
                nodes.row_data(1).expect("first references row"),
                pane_clip.clone(),
            )
            .is_none(),
        "the References scroll offset must cull the first row"
    );

    let (references_row, references_clip) = projector
        .transform(
            nodes.row_data(3).expect("second references row"),
            pane_clip.clone(),
        )
        .expect("the second References row enters after scroll");
    assert_eq!(references_row.frame.y, 18.0);
    assert!(references_row.hovered);
    assert_eq!(references_clip, frame(5.0, 27.0, 100.0, 60.0));

    let (used_by_row, _) = projector
        .transform(nodes.row_data(6).expect("first Used By row"), pane_clip)
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
        .transform(nodes.row_data(1).expect("header"), pane_clip.clone())
        .expect("header remains visible");
    assert_eq!(header.frame.y, 20.0);
    assert_eq!(header_clip, pane_clip);
    assert!(projector
        .transform(nodes.row_data(2).expect("first row"), pane_clip.clone())
        .is_none());
    let (second, row_clip) = projector
        .transform(nodes.row_data(3).expect("second row"), pane_clip.clone())
        .expect("second row scrolls into viewport");
    assert_eq!(second.frame.y, 44.0);
    assert!(second.hovered);
    assert_eq!(row_clip, frame(15.0, 51.0, 120.0, 46.0));

    let (preview, preview_clip) = projector
        .transform(nodes.row_data(4).expect("preview"), pane_clip.clone())
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
        .transform(
            nodes.row_data(3).expect("first tree row"),
            pane_clip.clone()
        )
        .is_none());
    let (row, row_clip) = projector
        .transform(nodes.row_data(4).expect("second tree row"), pane_clip)
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
        .transform(
            nodes.row_data(1).expect("first reference row"),
            pane_clip.clone(),
        )
        .is_none());

    let (row, clip) = projector
        .transform(
            nodes.row_data(3).expect("second reference row"),
            pane_clip.clone(),
        )
        .expect("second reference row enters after scroll");
    assert_eq!(row.frame.y, 18.0);
    assert!(row.hovered);
    assert_eq!(clip, frame(5.0, 27.0, 100.0, 60.0));

    let (label, _) = projector
        .transform(
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
            .transform(
                nodes.row_data(1).expect("first references row"),
                pane_clip.clone(),
            )
            .is_none(),
        "the References list scroll must not leave its first row visible"
    );
    let (references_row, _) = projector
        .transform(
            nodes.row_data(2).expect("second references row"),
            pane_clip.clone(),
        )
        .expect("second references row after its independent scroll");
    assert_eq!(references_row.frame.y, 18.0);
    assert!(references_row.hovered);

    let (used_by_row, _) = projector
        .transform(nodes.row_data(4).expect("first used-by row"), pane_clip)
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
        .transform(nodes.row_data(0).expect("grid"), pane_clip.clone())
        .expect("grid remains fixed");
    assert_eq!(grid.frame.y, 20.0);
    assert_eq!(grid_clip, pane_clip);

    let (card, card_clip) = projector
        .transform(nodes.row_data(3).expect("second card"), pane_clip.clone())
        .expect("second card remains visible after scroll");
    assert_eq!(card.frame.y, 12.0);
    assert!(card.hovered);
    assert_eq!(card_clip, frame(15.0, 27.0, 120.0, 80.0));

    let (band, _) = projector
        .transform(
            nodes.row_data(4).expect("second info band"),
            pane_clip.clone(),
        )
        .expect("second info band follows its card");
    assert_eq!(band.frame.y, 42.0);
    assert!(band.hovered);

    let (name, _) = projector
        .transform(nodes.row_data(5).expect("second name"), pane_clip)
        .expect("second name follows its card");
    assert_eq!(name.frame.y, 45.0);
    assert!(!name.hovered);
}

fn activity_model(nodes: Vec<ViewTemplateNodeData>) -> ModelRc<TemplatePaneNodeData> {
    host_model(nodes, AssetContentSurface::Activity)
}

fn browser_model(nodes: Vec<ViewTemplateNodeData>) -> ModelRc<TemplatePaneNodeData> {
    host_model(nodes, AssetContentSurface::Browser)
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
