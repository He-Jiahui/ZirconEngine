use super::route_asset_reference_hit;
use crate::ui::retained_host::host_contract::data::{
    AssetBrowserPaneData, AssetsActivityPaneData, FrameRect, PaneData, TemplateNodeFrameData,
    TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::native_pointer::routing::PanePointerTarget;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::asset_content_layout::{
    asset_content_paint_metadata, AssetContentPaintNodeInput, AssetContentSurface,
};

#[test]
fn browser_reference_lists_route_before_template_nodes_in_local_panel_coordinates() {
    let mut pane = PaneData {
        kind: "AssetBrowser".into(),
        ..PaneData::default()
    };
    pane.asset_browser = AssetBrowserPaneData {
        nodes: browser_nodes(vec![TemplatePaneNodeData {
            control_id: "AssetBrowserReferenceLeftScrollBody".into(),
            frame: TemplateNodeFrameData {
                x: 18.0,
                y: 62.0,
                width: 240.0,
                height: 112.0,
            },
            ..TemplatePaneNodeData::default()
        }]),
        ..AssetBrowserPaneData::default()
    };
    let body = FrameRect {
        x: 100.0,
        y: 80.0,
        width: 900.0,
        height: 620.0,
    };

    let route = route_asset_reference_hit(&pane, &body, 142.0, 154.0)
        .expect("reference panel should route as browser references");

    assert!(matches!(
        route.target,
        PanePointerTarget::AssetReference(ref mode, ref list_kind)
            if mode.as_str() == "browser" && list_kind.as_str() == "references"
    ));
    assert_eq!(route.local_x, 24.0);
    assert_eq!(route.local_y, 32.0);
    assert_eq!(route.width, 240.0);
    assert_eq!(route.height, 132.0);
}

#[test]
fn browser_used_by_routes_when_the_references_viewport_is_absent() {
    let mut pane = PaneData {
        kind: "AssetBrowser".into(),
        ..PaneData::default()
    };
    pane.asset_browser = AssetBrowserPaneData {
        nodes: browser_nodes(vec![TemplatePaneNodeData {
            control_id: "AssetBrowserReferenceRightScrollBody".into(),
            frame: TemplateNodeFrameData {
                x: 284.0,
                y: 62.0,
                width: 240.0,
                height: 112.0,
            },
            ..TemplatePaneNodeData::default()
        }]),
        ..AssetBrowserPaneData::default()
    };
    let body = FrameRect {
        x: 100.0,
        y: 80.0,
        width: 900.0,
        height: 620.0,
    };

    let route = route_asset_reference_hit(&pane, &body, 408.0, 154.0)
        .expect("Used By should route without a sibling references viewport");

    assert!(matches!(
        route.target,
        PanePointerTarget::AssetReference(ref mode, ref list_kind)
            if mode.as_str() == "browser" && list_kind.as_str() == "used_by"
    ));
    assert_eq!(route.local_x, 24.0);
    assert_eq!(route.local_y, 32.0);
}

#[test]
fn browser_used_by_routes_in_a_stacked_references_layout() {
    let mut pane = PaneData {
        kind: "AssetBrowser".into(),
        ..PaneData::default()
    };
    pane.asset_browser = AssetBrowserPaneData {
        nodes: browser_nodes(vec![TemplatePaneNodeData {
            control_id: "AssetBrowserReferenceRightScrollBody".into(),
            frame: TemplateNodeFrameData {
                x: 18.0,
                y: 210.0,
                width: 240.0,
                height: 112.0,
            },
            ..TemplatePaneNodeData::default()
        }]),
        ..AssetBrowserPaneData::default()
    };
    let body = FrameRect {
        x: 100.0,
        y: 80.0,
        width: 300.0,
        height: 620.0,
    };

    let route = route_asset_reference_hit(&pane, &body, 142.0, 302.0)
        .expect("stacked Used By viewport should route in its own local coordinates");

    assert!(matches!(
        route.target,
        PanePointerTarget::AssetReference(ref mode, ref list_kind)
            if mode.as_str() == "browser" && list_kind.as_str() == "used_by"
    ));
    assert_eq!(route.local_x, 24.0);
    assert_eq!(route.local_y, 32.0);
    assert_eq!(route.width, 240.0);
    assert_eq!(route.height, 132.0);
}

#[test]
fn activity_reference_lists_route_before_template_nodes_in_local_panel_coordinates() {
    let mut pane = PaneData {
        kind: "Assets".into(),
        ..PaneData::default()
    };
    pane.assets_activity = AssetsActivityPaneData {
        nodes: activity_nodes(vec![TemplatePaneNodeData {
            control_id: "AssetsActivityReferenceLeftScrollBody".into(),
            frame: TemplateNodeFrameData {
                x: 18.0,
                y: 62.0,
                width: 240.0,
                height: 112.0,
            },
            ..TemplatePaneNodeData::default()
        }]),
        ..AssetsActivityPaneData::default()
    };
    let body = FrameRect {
        x: 100.0,
        y: 80.0,
        width: 900.0,
        height: 620.0,
    };

    let route = route_asset_reference_hit(&pane, &body, 142.0, 154.0)
        .expect("reference panel should route as activity references");

    assert!(matches!(
        route.target,
        PanePointerTarget::AssetReference(ref mode, ref list_kind)
            if mode.as_str() == "activity" && list_kind.as_str() == "references"
    ));
    assert_eq!(route.local_x, 24.0);
    assert_eq!(route.local_y, 32.0);
    assert_eq!(route.width, 240.0);
    assert_eq!(route.height, 132.0);
}

fn browser_nodes(nodes: Vec<TemplatePaneNodeData>) -> ModelRc<TemplatePaneNodeData> {
    asset_nodes(nodes, AssetContentSurface::Browser)
}

fn activity_nodes(nodes: Vec<TemplatePaneNodeData>) -> ModelRc<TemplatePaneNodeData> {
    asset_nodes(nodes, AssetContentSurface::Activity)
}

fn asset_nodes(
    nodes: Vec<TemplatePaneNodeData>,
    surface: AssetContentSurface,
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
        surface,
    );
    ModelRc::with_metadata(nodes, metadata)
}
