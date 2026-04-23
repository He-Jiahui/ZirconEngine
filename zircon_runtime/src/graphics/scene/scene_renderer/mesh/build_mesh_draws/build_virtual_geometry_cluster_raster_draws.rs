use std::collections::HashMap;

use crate::graphics::types::{
    cluster_raster_draws_from_selections, ViewportRenderFrame, VirtualGeometryClusterRasterDraw,
};

pub(super) fn build_virtual_geometry_cluster_raster_draws(
    frame: &ViewportRenderFrame,
) -> HashMap<u64, Vec<VirtualGeometryClusterRasterDraw>> {
    frame
        .resolved_virtual_geometry_cluster_selections()
        .as_deref()
        .map(cluster_raster_draws_from_selections)
        .unwrap_or_default()
}
