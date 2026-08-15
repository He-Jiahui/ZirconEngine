use serde::{Deserialize, Serialize};
use zircon_runtime::core::framework::navigation::{
    NavAgentTickReport, NavMeshAsset, NavigationGizmoLink, NavigationGizmoSnapshot,
    NavigationGizmoTriangle,
};

pub const NAVIGATION_OVERLAY_FRAME_EVENT_ID: &str = "navigation.events.overlay_frame";
pub const NAVIGATION_OVERLAY_FRAME_PAYLOAD_SCHEMA: &str = "navigation.events.overlay_frame.v1";

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NavigationOverlayFrame {
    pub owner_generation: u64,
    pub nav_mesh: NavigationGizmoSnapshot,
    pub tick_report: NavAgentTickReport,
}

impl NavigationOverlayFrame {
    pub fn from_assets<'a>(
        owner_generation: u64,
        assets: impl IntoIterator<Item = &'a NavMeshAsset>,
        tick_report: NavAgentTickReport,
    ) -> Self {
        let mut nav_mesh = NavigationGizmoSnapshot::default();
        for asset in assets {
            nav_mesh
                .triangles
                .extend(asset.debug_triangles().into_iter().map(|triangle| {
                    NavigationGizmoTriangle {
                        vertices: triangle.vertices,
                        area: triangle.area,
                        tile: triangle.tile,
                    }
                }));
            nav_mesh
                .off_mesh_links
                .extend(asset.off_mesh_links.iter().map(|link| NavigationGizmoLink {
                    start: link.start,
                    end: link.end,
                    area: link.area,
                    bidirectional: link.bidirectional,
                }));
        }
        Self {
            owner_generation,
            nav_mesh,
            tick_report,
        }
    }
}
