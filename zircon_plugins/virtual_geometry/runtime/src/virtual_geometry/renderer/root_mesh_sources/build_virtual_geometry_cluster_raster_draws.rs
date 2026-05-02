use std::collections::HashMap;

use crate::virtual_geometry::types::{
    cluster_raster_draws_from_selections, VirtualGeometryClusterRasterDraw,
};
use zircon_runtime::graphics::ViewportRenderFrame;

pub(super) fn build_virtual_geometry_cluster_raster_draws(
    frame: &ViewportRenderFrame,
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
