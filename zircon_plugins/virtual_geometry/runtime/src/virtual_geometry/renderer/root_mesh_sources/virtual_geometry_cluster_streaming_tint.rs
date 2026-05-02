use crate::virtual_geometry::types::{
    VirtualGeometryClusterRasterDraw, VirtualGeometryPrepareClusterState,
};
use zircon_runtime::core::math::Vec4;

pub(super) fn virtual_geometry_cluster_streaming_tint(
    draw: VirtualGeometryClusterRasterDraw,
) -> Vec4 {
    let detail_boost = 1.0 + draw.lod_level as f32 * 0.08;
    match draw.state {
        VirtualGeometryPrepareClusterState::Resident => draw
            .resident_slot
            .map(|slot| {
                let slot_phase = slot.min(7) as f32;
                Vec4::new(
                    0.9 + slot_phase * 0.01,
                    (0.82 + slot_phase * 0.09) * detail_boost,
                    0.82 + slot_phase * 0.03,
                    1.0,
                )
            })
            .unwrap_or_else(|| Vec4::new(0.96, 1.04 * detail_boost, 0.9, 1.0)),
        VirtualGeometryPrepareClusterState::PendingUpload => {
            Vec4::new(0.78, 0.82 * detail_boost, 0.98, 1.0)
        }
        VirtualGeometryPrepareClusterState::Missing => Vec4::ZERO,
    }
}
