use std::collections::HashMap;

use crate::graphics::types::{
    cluster_raster_draws_from_selections, ViewportRenderFrame, VirtualGeometryClusterRasterDraw,
};

pub(super) fn build_virtual_geometry_cluster_raster_draws(
    frame: &ViewportRenderFrame,
) -> HashMap<u64, Vec<VirtualGeometryClusterRasterDraw>> {
    if let Some(selections) = frame.virtual_geometry_cluster_selections.as_ref() {
        return cluster_raster_draws_from_selections(selections);
    }

    frame
        .virtual_geometry_prepare
        .as_ref()
        .and_then(|prepare| {
            frame
                .extract
                .geometry
                .virtual_geometry
                .as_ref()
                .map(|extract| prepare.cluster_raster_draws(extract))
        })
        .unwrap_or_default()
}
