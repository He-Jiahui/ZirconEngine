use super::geometry::vertical_scrollbar_geometry;
use super::style::{workbench_scrollbar_metrics_from_host, WorkbenchScrollbarMetrics};
use super::{
    asset::{
        activity_asset_content_viewport_and_extent,
        activity_asset_reference_viewport_and_row_count, asset_tree_viewport_frame,
        browser_asset_reference_viewport_and_row_count,
    },
    draw_activity_asset_content_scrollbar, draw_activity_asset_reference_scrollbars,
    draw_browser_asset_reference_scrollbars,
};
use crate::ui::layouts::views::{ViewTemplateFrameData, ViewTemplateNodeData};
use crate::ui::retained_host::hierarchy_pointer::{
    current_hierarchy_row_metrics, hierarchy_content_height,
};
use crate::ui::retained_host::host_contract::data::{
    AssetBrowserPaneData, AssetsActivityPaneData, FrameRect, HostPaneInteractionStateData,
    PaneData, TemplateNodeFrameData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;
use crate::ui::workbench::asset_content_layout::{
    asset_content_paint_metadata, ActivityAssetReferenceListKind, AssetContentPaintNodeInput,
    AssetContentSurface, BrowserAssetReferenceListKind,
};

#[test]
fn scrollbar_metrics_match_unreal_starship_baseline() {
    let metrics = workbench_scrollbar_metrics_from_host(METRICS);

    assert_eq!(metrics.thickness, 8.0);
    assert_eq!(metrics.radius, 4.0);
    assert_eq!(metrics.track_inset, 1.0);
    assert_eq!(metrics.min_thumb_length, 24.0);
}

#[test]
fn vertical_scrollbar_thumb_uses_relative_scroll_extent() {
    let viewport = frame(10.0, 20.0, 100.0, 200.0);
    let geometry = vertical_scrollbar_geometry(&viewport, 300.0, 800.0, test_metrics())
        .expect("overflowing content should draw a scrollbar");

    assert_eq!(geometry.track, frame(101.0, 21.0, 8.0, 198.0));
    assert!((geometry.thumb.x - 101.0).abs() < 0.001);
    assert!((geometry.thumb.y - 95.25).abs() < 0.001);
    assert!((geometry.thumb.width - 8.0).abs() < 0.001);
    assert!((geometry.thumb.height - 49.5).abs() < 0.001);
}

#[test]
fn vertical_scrollbar_clamps_thumb_to_track_end() {
    let viewport = frame(0.0, 0.0, 40.0, 100.0);
    let geometry = vertical_scrollbar_geometry(&viewport, 999.0, 300.0, test_metrics())
        .expect("overflowing content should draw a scrollbar");

    assert!((geometry.thumb.y + geometry.thumb.height - 99.0).abs() < 0.001);
}

#[test]
fn vertical_scrollbar_is_absent_when_content_fits() {
    let viewport = frame(0.0, 0.0, 80.0, 200.0);

    assert!(vertical_scrollbar_geometry(&viewport, 0.0, 200.0, test_metrics()).is_none());
    assert!(vertical_scrollbar_geometry(&viewport, 0.0, 120.0, test_metrics()).is_none());
}

#[test]
fn hierarchy_content_extent_uses_row_metrics_not_pixel_coordinates() {
    let metrics = current_hierarchy_row_metrics();
    assert_eq!(hierarchy_content_height(0, metrics), 0.0);
    assert_eq!(
        hierarchy_content_height(3, metrics),
        metrics.row_y * 2.0 + 3.0 * metrics.row_height + 2.0 * metrics.row_gap
    );
}

#[test]
fn asset_tree_viewport_tracks_pointer_tree_header_formula() {
    let body = frame(20.0, 30.0, 220.0, 260.0);
    let viewport = asset_tree_viewport_frame(&body);

    assert_eq!(viewport.x, 20.0);
    assert_eq!(viewport.y, 79.0);
    assert_eq!(viewport.width, 220.0);
    assert_eq!(viewport.height, 211.0);
}

#[test]
fn activity_asset_content_scrollbar_uses_projected_panel_viewport_and_extent() {
    let pane = activity_content_pane(240.0);
    let body = frame(20.0, 30.0, 140.0, 120.0);
    let (viewport, extent) =
        activity_asset_content_viewport_and_extent(&pane.assets_activity.nodes, &body)
            .expect("activity content viewport");

    assert_eq!(viewport, frame(30.0, 50.0, 100.0, 80.0));
    assert_eq!(extent, 240.0);

    let metrics = workbench_scrollbar_metrics_from_host(METRICS);
    let top_geometry = vertical_scrollbar_geometry(&viewport, 0.0, extent, metrics)
        .expect("overflowing Activity content should have a top thumb");
    let scrolled_geometry = vertical_scrollbar_geometry(&viewport, 60.0, extent, metrics)
        .expect("overflowing Activity content should have a scrolled thumb");
    assert_eq!(scrolled_geometry.track, top_geometry.track);
    assert_eq!(scrolled_geometry.thumb.height, top_geometry.thumb.height);
    assert!(
        scrolled_geometry.thumb.y > top_geometry.thumb.y,
        "stored Activity content scroll must move the shared thumb down the track"
    );

    let mut pixels = HostRgbaFrame::filled(180, 180, [0, 0, 0, 255]);
    assert!(draw_activity_asset_content_scrollbar(
        &mut pixels,
        &pane,
        &body,
        &frame(0.0, 0.0, 180.0, 180.0),
        &HostPaneInteractionStateData {
            activity_asset_content_scroll_px: 60.0,
            activity_asset_content_hovered_index: 2,
            ..HostPaneInteractionStateData::default()
        },
    ));
}

#[test]
fn activity_asset_content_scrollbar_is_absent_for_empty_and_fitting_content() {
    let body = frame(20.0, 30.0, 140.0, 120.0);
    let clip = frame(0.0, 0.0, 180.0, 180.0);
    for extent in [0.0, 80.0] {
        let pane = activity_content_pane(extent);
        let mut pixels = HostRgbaFrame::filled(180, 180, [0, 0, 0, 255]);
        assert!(!draw_activity_asset_content_scrollbar(
            &mut pixels,
            &pane,
            &body,
            &clip,
            &HostPaneInteractionStateData::default(),
        ));
    }
}

#[test]
fn browser_reference_scrollbars_paint_each_list_with_independent_extents_and_scroll() {
    let pane = browser_reference_pane(3, 4);
    let body = frame(20.0, 30.0, 240.0, 240.0);
    let clip = frame(0.0, 0.0, 300.0, 300.0);
    let references = browser_asset_reference_viewport_and_row_count(
        &pane.asset_browser.nodes,
        &body,
        BrowserAssetReferenceListKind::References,
    )
    .expect("references viewport");
    let used_by = browser_asset_reference_viewport_and_row_count(
        &pane.asset_browser.nodes,
        &body,
        BrowserAssetReferenceListKind::UsedBy,
    )
    .expect("Used By viewport");
    assert_eq!(references.1, 3);
    assert_eq!(used_by.1, 4);

    let mut pixels = HostRgbaFrame::filled(300, 300, [0, 0, 0, 255]);
    assert!(draw_browser_asset_reference_scrollbars(
        &mut pixels,
        &pane,
        &body,
        &clip,
        &HostPaneInteractionStateData {
            browser_asset_references_scroll_px: 24.0,
            browser_asset_references_hovered_index: 1,
            browser_asset_used_by_scroll_px: 52.0,
            browser_asset_used_by_hovered_index: 2,
            ..HostPaneInteractionStateData::default()
        },
    ));

    let bytes = pixels.into_bytes();
    assert_ne!(pixel_at(&bytes, 300, 122, 76), [0, 0, 0, 255]);
    assert_ne!(pixel_at(&bytes, 300, 242, 176), [0, 0, 0, 255]);
}

#[test]
fn activity_reference_scrollbars_paint_each_list_with_independent_extents_and_scroll() {
    let pane = activity_reference_pane(3, 4);
    let body = frame(20.0, 30.0, 240.0, 240.0);
    let clip = frame(0.0, 0.0, 300.0, 300.0);
    let references = activity_asset_reference_viewport_and_row_count(
        &pane.assets_activity.nodes,
        &body,
        ActivityAssetReferenceListKind::References,
    )
    .expect("activity References viewport");
    let used_by = activity_asset_reference_viewport_and_row_count(
        &pane.assets_activity.nodes,
        &body,
        ActivityAssetReferenceListKind::UsedBy,
    )
    .expect("activity Used By viewport");
    assert_eq!(references.1, 3);
    assert_eq!(used_by.1, 4);

    let mut pixels = HostRgbaFrame::filled(300, 300, [0, 0, 0, 255]);
    assert!(draw_activity_asset_reference_scrollbars(
        &mut pixels,
        &pane,
        &body,
        &clip,
        &HostPaneInteractionStateData {
            activity_asset_references_scroll_px: 24.0,
            activity_asset_references_hovered_index: 1,
            activity_asset_used_by_scroll_px: 52.0,
            activity_asset_used_by_hovered_index: 2,
            ..HostPaneInteractionStateData::default()
        },
    ));

    let bytes = pixels.into_bytes();
    assert_ne!(pixel_at(&bytes, 300, 122, 76), [0, 0, 0, 255]);
    assert_ne!(pixel_at(&bytes, 300, 242, 176), [0, 0, 0, 255]);
}

fn test_metrics() -> WorkbenchScrollbarMetrics {
    WorkbenchScrollbarMetrics {
        thickness: 8.0,
        radius: 4.0,
        track_inset: 1.0,
        min_thumb_length: 24.0,
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

fn activity_content_pane(content_extent: f32) -> PaneData {
    let view_nodes = vec![ViewTemplateNodeData {
        control_id: "AssetsActivityContentPanel".into(),
        value_number: content_extent,
        frame: ViewTemplateFrameData {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 80.0,
        },
        ..ViewTemplateNodeData::default()
    }];
    let metadata = asset_content_paint_metadata(
        view_nodes.iter().map(|node| {
            AssetContentPaintNodeInput::new(
                node.control_id.as_str(),
                node.frame.x,
                node.frame.y,
                node.frame.width,
                node.frame.height,
                node.value_number,
            )
        }),
        AssetContentSurface::Activity,
    );
    let nodes = crate::ui::retained_host::primitives::ModelRc::with_metadata(view_nodes, metadata)
        .map_preserving_metadata(|node| TemplatePaneNodeData {
            control_id: node.control_id.clone(),
            value_number: node.value_number,
            frame: TemplateNodeFrameData {
                x: node.frame.x,
                y: node.frame.y,
                width: node.frame.width,
                height: node.frame.height,
            },
            ..TemplatePaneNodeData::default()
        });

    PaneData {
        kind: "Assets".into(),
        assets_activity: AssetsActivityPaneData { nodes },
        ..PaneData::default()
    }
}

fn browser_reference_pane(reference_count: usize, used_by_count: usize) -> PaneData {
    let mut view_nodes = vec![
        view_node(
            "AssetBrowserReferenceLeftScrollBody",
            10.0,
            20.0,
            100.0,
            60.0,
        ),
        view_node(
            "AssetBrowserReferenceRightScrollBody",
            130.0,
            120.0,
            100.0,
            60.0,
        ),
    ];
    for index in 1..=reference_count {
        view_nodes.push(view_node(
            format!("AssetBrowserReferenceLeftRowPanel{index:02}").as_str(),
            10.0,
            20.0 + (index - 1) as f32 * 38.0,
            96.0,
            34.0,
        ));
    }
    for index in 1..=used_by_count {
        view_nodes.push(view_node(
            format!("AssetBrowserReferenceRightRowPanel{index:02}").as_str(),
            130.0,
            120.0 + (index - 1) as f32 * 38.0,
            96.0,
            34.0,
        ));
    }
    let metadata = asset_content_paint_metadata(
        view_nodes.iter().map(|node| {
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
    );
    let nodes = crate::ui::retained_host::primitives::ModelRc::with_metadata(view_nodes, metadata)
        .map_preserving_metadata(|node| TemplatePaneNodeData {
            control_id: node.control_id.clone(),
            value_number: node.value_number,
            frame: TemplateNodeFrameData {
                x: node.frame.x,
                y: node.frame.y,
                width: node.frame.width,
                height: node.frame.height,
            },
            ..TemplatePaneNodeData::default()
        });

    PaneData {
        kind: "AssetBrowser".into(),
        asset_browser: AssetBrowserPaneData { nodes },
        ..PaneData::default()
    }
}

fn activity_reference_pane(reference_count: usize, used_by_count: usize) -> PaneData {
    let mut view_nodes = vec![
        view_node(
            "AssetsActivityReferenceLeftScrollBody",
            10.0,
            20.0,
            100.0,
            60.0,
        ),
        view_node(
            "AssetsActivityReferenceRightScrollBody",
            130.0,
            120.0,
            100.0,
            60.0,
        ),
    ];
    for index in 1..=reference_count {
        view_nodes.push(view_node(
            format!("AssetsActivityReferenceLeftRowPanel{index:02}").as_str(),
            10.0,
            20.0 + (index - 1) as f32 * 38.0,
            96.0,
            34.0,
        ));
    }
    for index in 1..=used_by_count {
        view_nodes.push(view_node(
            format!("AssetsActivityReferenceRightRowPanel{index:02}").as_str(),
            130.0,
            120.0 + (index - 1) as f32 * 38.0,
            96.0,
            34.0,
        ));
    }
    let metadata = asset_content_paint_metadata(
        view_nodes.iter().map(|node| {
            AssetContentPaintNodeInput::new(
                node.control_id.as_str(),
                node.frame.x,
                node.frame.y,
                node.frame.width,
                node.frame.height,
                node.value_number,
            )
        }),
        AssetContentSurface::Activity,
    );
    let nodes = crate::ui::retained_host::primitives::ModelRc::with_metadata(view_nodes, metadata)
        .map_preserving_metadata(|node| TemplatePaneNodeData {
            control_id: node.control_id.clone(),
            value_number: node.value_number,
            frame: TemplateNodeFrameData {
                x: node.frame.x,
                y: node.frame.y,
                width: node.frame.width,
                height: node.frame.height,
            },
            ..TemplatePaneNodeData::default()
        });

    PaneData {
        kind: "Assets".into(),
        assets_activity: AssetsActivityPaneData { nodes },
        ..PaneData::default()
    }
}

fn view_node(control_id: &str, x: f32, y: f32, width: f32, height: f32) -> ViewTemplateNodeData {
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

fn pixel_at(bytes: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * width as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}
