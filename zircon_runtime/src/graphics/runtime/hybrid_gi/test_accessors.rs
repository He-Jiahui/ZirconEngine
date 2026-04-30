use super::{
    declarations::HybridGiRuntimeProbeSceneData, HybridGiProbeResidencyState,
    HybridGiProbeUpdateRequest, HybridGiRuntimeState, HybridGiScenePrepareResourceSamples,
};
use crate::core::math::Vec3;

const SIGNED_POSITION_SCALE: f32 = 64.0;
const SIGNED_POSITION_BIAS: i32 = 2048;
const POSITIVE_RADIUS_SCALE: f32 = 96.0;

impl HybridGiScenePrepareResourceSamples
    for crate::graphics::scene::HybridGiScenePrepareResourcesSnapshot
{
    fn atlas_slot_rgba_sample(&self, atlas_slot_id: u32) -> Option<[u8; 4]> {
        self.atlas_slot_rgba_sample(atlas_slot_id)
    }

    fn capture_slot_rgba_sample(&self, capture_slot_id: u32) -> Option<[u8; 4]> {
        self.capture_slot_rgba_sample(capture_slot_id)
    }
}

impl HybridGiRuntimeState {
    pub(crate) fn probe_slot(&self, probe_id: u32) -> Option<u32> {
        self.resident_probe_slots()
            .find_map(|(resident_probe_id, slot)| (resident_probe_id == probe_id).then_some(slot))
    }

    pub(crate) fn probe_residency(&self, probe_id: u32) -> Option<HybridGiProbeResidencyState> {
        if self.has_resident_probe(probe_id) {
            return Some(HybridGiProbeResidencyState::Resident);
        }
        if self.has_pending_probe(probe_id) {
            return Some(HybridGiProbeResidencyState::PendingUpdate);
        }
        None
    }

    pub(crate) fn pending_updates(&self) -> Vec<HybridGiProbeUpdateRequest> {
        self.pending_update_requests().to_vec()
    }

    pub(crate) fn scheduled_trace_regions(&self) -> Vec<u32> {
        self.scheduled_trace_region_ids().to_vec()
    }

    pub(crate) fn scene_card_ids(&self) -> Vec<u32> {
        self.scene_representation().card_ids()
    }

    pub(crate) fn scene_resident_page_ids(&self) -> Vec<u32> {
        self.scene_representation()
            .surface_cache()
            .resident_page_ids()
    }

    pub(crate) fn scene_dirty_page_ids(&self) -> Vec<u32> {
        self.scene_representation().surface_cache().dirty_page_ids()
    }

    pub(crate) fn scene_invalidated_page_ids(&self) -> Vec<u32> {
        self.scene_representation()
            .surface_cache()
            .invalidated_page_ids()
    }

    pub(crate) fn scene_feedback_card_ids(&self) -> Vec<u32> {
        self.scene_representation()
            .surface_cache()
            .feedback_card_ids()
    }

    pub(crate) fn scene_page_table_entries(&self) -> Vec<(u32, u32)> {
        self.scene_representation()
            .surface_cache()
            .page_table_entries()
    }

    pub(crate) fn scene_capture_slot_entries(&self) -> Vec<(u32, u32)> {
        self.scene_representation()
            .surface_cache()
            .capture_slot_entries()
    }

    pub(crate) fn scene_card_capture_requests(&self) -> Vec<(u32, u32, u32, u32, [f32; 3], f32)> {
        self.scene_representation().card_capture_requests()
    }

    pub(crate) fn scene_surface_cache_page_contents(
        &self,
    ) -> Vec<(u32, u32, u32, u32, [u8; 4], [u8; 4])> {
        self.scene_representation().surface_cache().page_contents()
    }

    pub(crate) fn scene_resident_clipmap_ids(&self) -> Vec<u32> {
        self.scene_representation()
            .voxel_scene()
            .resident_clipmap_ids()
    }

    pub(crate) fn scene_dirty_clipmap_ids(&self) -> Vec<u32> {
        self.scene_representation()
            .voxel_scene()
            .dirty_clipmap_ids()
    }

    pub(crate) fn scene_invalidated_clipmap_ids(&self) -> Vec<u32> {
        self.scene_representation()
            .voxel_scene()
            .invalidated_clipmap_ids()
    }

    pub(crate) fn scene_clipmap_descriptors(&self) -> Vec<(u32, [f32; 3], f32)> {
        self.scene_representation()
            .voxel_scene()
            .clipmap_descriptors()
    }

    pub(crate) fn apply_scene_prepare_resources_for_test(
        &mut self,
        resources: &dyn HybridGiScenePrepareResourceSamples,
    ) {
        self.apply_scene_prepare_resources(resources);
    }

    pub(crate) fn seed_runtime_probe_scene_data_for_test(
        &mut self,
        probes: impl IntoIterator<Item = (u32, Vec3, f32, Option<u32>, u32)>,
    ) {
        self.probe_scene_data_mut().clear();
        self.probe_parent_probes_mut().clear();
        self.probe_ray_budgets_mut().clear();
        for (probe_id, position, radius, parent_probe_id, ray_budget) in probes {
            self.probe_scene_data_mut().insert(
                probe_id,
                HybridGiRuntimeProbeSceneData::new(
                    quantized_signed(position.x),
                    quantized_signed(position.y),
                    quantized_signed(position.z),
                    quantized_positive(radius, POSITIVE_RADIUS_SCALE),
                ),
            );
            self.probe_ray_budgets_mut().insert(probe_id, ray_budget);
            if let Some(parent_probe_id) = parent_probe_id {
                self.probe_parent_probes_mut()
                    .insert(probe_id, parent_probe_id);
            }
        }
    }

    pub(crate) fn evictable_probes(&self) -> Vec<u32> {
        self.evictable_probe_ids().to_vec()
    }

    pub(crate) fn apply_evictions(&mut self, probe_ids: impl IntoIterator<Item = u32>) {
        for probe_id in probe_ids {
            if let Some(slot) = self.remove_resident_probe_slot(probe_id) {
                self.insert_free_slot(slot);
            }
        }
        self.retain_resident_evictable_probes();
    }

    pub(crate) fn fulfill_updates(&mut self, probe_ids: impl IntoIterator<Item = u32>) {
        for probe_id in probe_ids {
            if !self.remove_pending_probe(probe_id) {
                continue;
            }

            self.retain_pending_update_requests(|update| update.probe_id() != probe_id);
            self.promote_to_resident(probe_id);
        }
    }
}

fn quantized_signed(value: f32) -> u32 {
    ((value * SIGNED_POSITION_SCALE).round() as i32).wrapping_add(SIGNED_POSITION_BIAS) as u32
}

fn quantized_positive(value: f32, scale: f32) -> u32 {
    (value.max(0.0) * scale).round() as u32
}
