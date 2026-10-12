use std::collections::HashMap;

use crate::virtual_geometry::renderer::VirtualGeometryRenderFrame;
use crate::virtual_geometry::types::{
    cluster_raster_draws_from_selections, VirtualGeometryClusterRasterDraw,
};

pub(super) fn build_virtual_geometry_cluster_raster_draws(
    frame: &VirtualGeometryRenderFrame,
) -> HashMap<u64, Vec<VirtualGeometryClusterRasterDraw>> {
    frame
        .extract
        .geometry
        .virtual_geometry
        .as_ref()
        .map(|_| &[][..])
        .map(cluster_raster_draws_from_selections)
        .unwrap_or_default()
}
