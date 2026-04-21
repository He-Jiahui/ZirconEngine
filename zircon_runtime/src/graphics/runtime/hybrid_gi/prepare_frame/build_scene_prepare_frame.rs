use crate::graphics::types::{
    HybridGiPrepareCardCaptureRequest, HybridGiPrepareVoxelClipmap, HybridGiScenePrepareFrame,
};

use super::super::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(crate) fn build_scene_prepare_frame(&self) -> HybridGiScenePrepareFrame {
        HybridGiScenePrepareFrame {
            card_capture_requests: self
                .scene_representation
                .card_capture_requests
                .iter()
                .map(|request| HybridGiPrepareCardCaptureRequest {
                    card_id: request.card_id,
                    page_id: request.page_id,
                    atlas_slot_id: request.atlas_slot_id,
                    capture_slot_id: request.capture_slot_id,
                    bounds_center: request.bounds_center,
                    bounds_radius: request.bounds_radius,
                })
                .collect(),
            voxel_clipmaps: self
                .scene_representation
                .voxel_scene
                .clipmap_descriptors_snapshot()
                .into_iter()
                .map(
                    |(clipmap_id, center, half_extent)| HybridGiPrepareVoxelClipmap {
                        clipmap_id,
                        center,
                        half_extent,
                    },
                )
                .collect(),
            voxel_cells: self.scene_representation.voxel_scene.voxel_cells_snapshot(),
        }
    }
}
