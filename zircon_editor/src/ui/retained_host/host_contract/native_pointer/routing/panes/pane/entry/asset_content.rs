use crate::ui::retained_host::host_contract::data::{FrameRect, PaneData};
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::asset_content_layout::{
    ACTIVITY_CONTENT_PANEL_CONTROL_ID, BROWSER_CONTENT_TABLE_CONTROL_ID,
    BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID,
};

use super::super::super::super::{geometry::contains, PanePointerRoute, PanePointerTarget};

const ACTIVITY_ASSET_SURFACE_MODE: &str = "activity";
const BROWSER_ASSET_SURFACE_MODE: &str = "browser";

pub(super) fn route_asset_content_hit(
    pane: &PaneData,
    body: &FrameRect,
    x: f32,
    y: f32,
) -> Option<PanePointerRoute> {
    let (nodes, panel_control_ids, surface_mode) = match pane.kind.as_str() {
        "Assets" => (
            &pane.assets_activity.nodes,
            &[ACTIVITY_CONTENT_PANEL_CONTROL_ID][..],
            ACTIVITY_ASSET_SURFACE_MODE,
        ),
        "AssetBrowser" => (
            &pane.asset_browser.nodes,
            &[
                BROWSER_CONTENT_THUMBNAIL_GRID_CONTROL_ID,
                BROWSER_CONTENT_TABLE_CONTROL_ID,
            ][..],
            BROWSER_ASSET_SURFACE_MODE,
        ),
        _ => return None,
    };
    let panel = panel_control_ids
        .iter()
        .find_map(|control_id| find_panel(nodes, control_id))?;
    let panel_frame = FrameRect {
        x: body.x + panel.frame.x,
        y: body.y + panel.frame.y,
        width: panel.frame.width.max(0.0),
        height: panel.frame.height.max(0.0),
    };
    if !contains(&panel_frame, x, y) {
        return None;
    }

    Some(PanePointerRoute::new(
        PanePointerTarget::AssetContent(surface_mode.into()),
        &panel_frame,
        x,
        y,
    ))
}

fn find_panel(
    nodes: &ModelRc<crate::ui::retained_host::host_contract::data::TemplatePaneNodeData>,
    panel_control_id: &str,
) -> Option<crate::ui::retained_host::host_contract::data::TemplatePaneNodeData> {
    (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .find(|node| node.control_id.rsplit('/').next() == Some(panel_control_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::layouts::common::model_rc;
    use crate::ui::retained_host::host_contract::data::{
        AssetBrowserPaneData, TemplateNodeFrameData, TemplatePaneNodeData,
    };

    #[test]
    fn browser_table_routes_to_shared_browser_content_surface() {
        let mut pane = PaneData {
            kind: "AssetBrowser".into(),
            ..PaneData::default()
        };
        pane.asset_browser = AssetBrowserPaneData {
            nodes: model_rc(vec![TemplatePaneNodeData {
                control_id: "AssetBrowserAssetTablePanel".into(),
                frame: TemplateNodeFrameData {
                    x: 40.0,
                    y: 60.0,
                    width: 240.0,
                    height: 160.0,
                },
                ..TemplatePaneNodeData::default()
            }]),
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
            nodes: model_rc(vec![TemplatePaneNodeData {
                control_id: "AssetBrowserThumbGridPanel".into(),
                frame: TemplateNodeFrameData {
                    x: 40.0,
                    y: 60.0,
                    width: 320.0,
                    height: 220.0,
                },
                ..TemplatePaneNodeData::default()
            }]),
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
            nodes: model_rc(vec![
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
}
