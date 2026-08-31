use crate::ui::retained_host::host_contract::data::{FrameRect, PaneData};
use crate::ui::workbench::asset_content_layout::{AssetContentPaintMetadata, AssetContentSurface};

use super::super::super::super::{
    geometry::contains, PaneAssetSurface, PanePointerRoute, PanePointerTarget,
};

pub(super) fn route_asset_content_hit(
    pane: &PaneData,
    body: &FrameRect,
    x: f32,
    y: f32,
) -> Option<PanePointerRoute> {
    let (nodes, surface, surface_mode) = match pane.kind.as_str() {
        "Assets" => (
            &pane.assets_activity.nodes,
            AssetContentSurface::Activity,
            PaneAssetSurface::Activity,
        ),
        "AssetBrowser" => (
            &pane.asset_browser.nodes,
            AssetContentSurface::Browser,
            PaneAssetSurface::Browser,
        ),
        _ => return None,
    };
    let metadata = nodes.metadata::<AssetContentPaintMetadata>()?;
    if metadata.surface() != surface {
        return None;
    }
    let panel = metadata.content_panel()?;
    let panel_frame = FrameRect {
        x: body.x + panel.x,
        y: body.y + panel.y,
        width: panel.width.max(0.0),
        height: panel.height.max(0.0),
    };
    if !contains(&panel_frame, x, y) {
        return None;
    }

    Some(PanePointerRoute::new(
        PanePointerTarget::AssetContent(surface_mode),
        &panel_frame,
        x,
        y,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::data::{
        AssetBrowserPaneData, TemplateNodeFrameData, TemplatePaneNodeData,
    };
    use crate::ui::retained_host::primitives::ModelRc;
    use crate::ui::workbench::asset_content_layout::{
        asset_content_paint_metadata, AssetContentPaintNodeInput,
    };

    #[test]
    fn browser_table_routes_to_shared_browser_content_surface() {
        let mut pane = PaneData {
            kind: "AssetBrowser".into(),
            ..PaneData::default()
        };
        pane.asset_browser = AssetBrowserPaneData {
            nodes: browser_nodes(vec![TemplatePaneNodeData {
                control_id: "AssetBrowserAssetTablePanel".into(),
                frame: TemplateNodeFrameData {
                    x: 40.0,
                    y: 60.0,
                    width: 240.0,
                    height: 160.0,
                },
                ..TemplatePaneNodeData::default()
            }]),
            ..AssetBrowserPaneData::default()
        };
        let body = FrameRect {
            x: 100.0,
            y: 80.0,
            width: 500.0,
            height: 400.0,
        };

        let route = route_asset_content_hit(&pane, &body, 160.0, 160.0)
            .expect("browser table should route through shared asset content input");

        assert!(matches!(
            route.target,
            PanePointerTarget::AssetContent(ref mode) if mode.as_str() == "browser"
        ));
        assert_eq!(route.local_x, 20.0);
        assert_eq!(route.local_y, 20.0);
        assert_eq!(route.width, 240.0);
        assert_eq!(route.height, 160.0);
    }

    #[test]
    fn browser_thumbnail_grid_routes_to_shared_browser_content_surface() {
        let mut pane = PaneData {
            kind: "AssetBrowser".into(),
            ..PaneData::default()
        };
        pane.asset_browser = AssetBrowserPaneData {
            nodes: browser_nodes(vec![TemplatePaneNodeData {
                control_id: "AssetBrowserThumbGridPanel".into(),
                frame: TemplateNodeFrameData {
                    x: 40.0,
                    y: 60.0,
                    width: 320.0,
                    height: 220.0,
                },
                ..TemplatePaneNodeData::default()
            }]),
            ..AssetBrowserPaneData::default()
        };
        let body = FrameRect {
            x: 100.0,
            y: 80.0,
            width: 500.0,
            height: 400.0,
        };

        let route = route_asset_content_hit(&pane, &body, 200.0, 180.0)
            .expect("browser thumbnail grid should route through shared asset content input");

        assert!(matches!(
            route.target,
            PanePointerTarget::AssetContent(ref mode) if mode.as_str() == "browser"
        ));
        assert_eq!(route.local_x, 60.0);
        assert_eq!(route.local_y, 40.0);
        assert_eq!(route.width, 320.0);
        assert_eq!(route.height, 220.0);
    }

    #[test]
    fn browser_thumbnail_grid_takes_priority_over_retained_table_nodes() {
        let mut pane = PaneData {
            kind: "AssetBrowser".into(),
            ..PaneData::default()
        };
        pane.asset_browser = AssetBrowserPaneData {
            nodes: browser_nodes(vec![
                TemplatePaneNodeData {
                    control_id: "AssetBrowserAssetTablePanel".into(),
                    frame: TemplateNodeFrameData {
                        x: 0.0,
                        y: 0.0,
                        width: 500.0,
                        height: 400.0,
                    },
                    ..TemplatePaneNodeData::default()
                },
                TemplatePaneNodeData {
                    control_id: "AssetBrowserThumbGridPanel".into(),
                    frame: TemplateNodeFrameData {
                        x: 20.0,
                        y: 60.0,
                        width: 320.0,
                        height: 220.0,
                    },
                    ..TemplatePaneNodeData::default()
                },
            ]),
            ..AssetBrowserPaneData::default()
        };
        let body = FrameRect {
            x: 100.0,
            y: 80.0,
            width: 500.0,
            height: 400.0,
        };

        let route = route_asset_content_hit(&pane, &body, 180.0, 180.0)
            .expect("thumbnail grid should win over retained table nodes");

        assert_eq!(route.local_x, 60.0);
        assert_eq!(route.local_y, 40.0);
        assert_eq!(route.width, 320.0);
        assert_eq!(route.height, 220.0);
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
}
