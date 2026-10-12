use super::route_browser_asset_tree_hit;
use crate::ui::retained_host::host_contract::data::{
    AssetBrowserPaneData, FrameRect, PaneData, TemplateNodeFrameData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::native_pointer::routing::PanePointerTarget;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::asset_content_layout::{
    asset_content_paint_metadata, AssetContentPaintNodeInput, AssetContentSurface,
};

#[test]
fn browser_sources_panel_routes_tree_input_in_local_panel_coordinates() {
    let mut pane = PaneData {
        kind: "AssetBrowser".into(),
        ..PaneData::default()
    };
    pane.asset_browser = AssetBrowserPaneData {
        nodes: browser_nodes(vec![
            TemplatePaneNodeData {
                control_id: "AssetBrowserSourcesPanel".into(),
                frame: TemplateNodeFrameData {
                    x: 18.0,
                    y: 42.0,
                    width: 152.0,
                    height: 430.0,
                },
                ..TemplatePaneNodeData::default()
            },
            TemplatePaneNodeData {
                control_id: "AssetBrowserSourcesScrollBody".into(),
                frame: TemplateNodeFrameData {
                    x: 18.0,
                    y: 91.0,
                    width: 152.0,
                    height: 381.0,
                },
                ..TemplatePaneNodeData::default()
            },
        ]),
        ..AssetBrowserPaneData::default()
    };
    let body = FrameRect {
        x: 100.0,
        y: 80.0,
        width: 900.0,
        height: 620.0,
    };

    let route = route_browser_asset_tree_hit(&pane, &body, 142.0, 154.0)
        .expect("sources panel should route as the browser tree surface");

    assert!(matches!(
        route.target,
        PanePointerTarget::AssetTree(ref mode) if mode.as_str() == "browser"
    ));
    assert_eq!(route.local_x, 24.0);
    assert_eq!(route.local_y, 32.0);
    assert_eq!(route.width, 152.0);
    assert_eq!(route.height, 430.0);
}

fn browser_nodes(nodes: Vec<TemplatePaneNodeData>) -> ModelRc<TemplatePaneNodeData> {
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
    );
    ModelRc::with_metadata(nodes, metadata)
}
