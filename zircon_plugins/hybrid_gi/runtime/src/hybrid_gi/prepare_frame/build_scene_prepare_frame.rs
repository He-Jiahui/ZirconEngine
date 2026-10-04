use crate::hybrid_gi::{
    HybridGiPrepareCardCaptureRequest, HybridGiPrepareSurfaceCachePageContent,
    HybridGiPrepareVoxelClipmap, HybridGiScenePrepareFrame,
};

use super::super::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    // 这里取的是场景表示的当前快照；GPU 完成后的资源在下一次准备帧才可见。
    pub(crate) fn build_scene_prepare_frame(&self) -> HybridGiScenePrepareFrame {
        let scene_representation = self.scene_representation();
        let card_bounds_by_owner_card_id = scene_representation.card_bounds_by_id();
        HybridGiScenePrepareFrame {
            card_capture_requests: self
                .scene_representation()
                .card_capture_request_descriptors()
                .iter()
                .map(|request| HybridGiPrepareCardCaptureRequest {
                    card_id: request.card_id(),
                    page_id: request.page_id(),
                    atlas_slot_id: request.atlas_slot_id(),
                    capture_slot_id: request.capture_slot_id(),
                    bounds_center: request.bounds_center(),
                    bounds_radius: request.bounds_radius(),
                })
                .collect(),
            surface_cache_page_contents: self
                .scene_representation()
                .surface_cache()
                .page_contents_snapshot()
                .into_iter()
                .filter_map(
                    |(
                        page_id,
                        owner_card_id,
                        atlas_slot_id,
                        capture_slot_id,
                        atlas_sample_rgba,
                        capture_sample_rgba,
                    )| {
                        let (bounds_center, bounds_radius) =
                            card_bounds_by_owner_card_id.get(&owner_card_id).copied()?;
                        Some(HybridGiPrepareSurfaceCachePageContent {
                            page_id,
                            owner_card_id,
                            atlas_slot_id,
                            capture_slot_id,
                            bounds_center,
                            bounds_radius,
                            atlas_sample_rgba,
                            capture_sample_rgba,
                        })
                    },
                )
                .collect(),
            voxel_clipmaps: self
                .scene_representation()
                .voxel_scene()
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
            voxel_cells: self
                .scene_representation()
                .voxel_scene()
                .voxel_cells_snapshot(),
            card_owner_stable_instance_keys: scene_representation.card_owner_stable_instance_keys(),
            radiance_cache_bootstrap_updates: scene_representation
                .radiance_cache_gpu_bootstrap_updates(),
            radiance_cache_updates: scene_representation.radiance_cache_gpu_updates(),
            radiance_cache_consumes: scene_representation.radiance_cache_gpu_consumes(),
        }
    }
}

#[cfg(test)]
#[path = "tests/build_scene_prepare_frame.rs"]
mod tests;
