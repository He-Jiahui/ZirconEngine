use std::collections::BTreeSet;

use crate::core::framework::render::{
    RenderDirectionalLightSnapshot, RenderHybridGiExtract, RenderMeshSnapshot,
    RenderPointLightSnapshot, RenderSpotLightSnapshot,
};

use super::HybridGiRuntimeState;

const SIGNED_POSITION_SCALE: f32 = 64.0;
const SIGNED_POSITION_BIAS: i32 = 2048;
const POSITIVE_RADIUS_SCALE: f32 = 96.0;
const POSITIVE_COVERAGE_SCALE: f32 = 128.0;

impl HybridGiRuntimeState {
    pub(crate) fn register_scene_extract(
        &mut self,
        extract: Option<&RenderHybridGiExtract>,
        meshes: &[RenderMeshSnapshot],
        directional_lights: &[RenderDirectionalLightSnapshot],
        point_lights: &[RenderPointLightSnapshot],
        spot_lights: &[RenderSpotLightSnapshot],
    ) {
        self.register_extract(extract);
        if extract.is_some() {
            self.scene_representation.synchronize_scene(
                meshes,
                directional_lights,
                point_lights,
                spot_lights,
            );
        }
    }

    pub(crate) fn register_extract(&mut self, extract: Option<&RenderHybridGiExtract>) {
        self.evictable_probes.clear();
        self.scheduled_trace_regions.clear();
        self.current_requested_probe_ids.clear();

        let Some(extract) = extract else {
            *self = Self::default();
            return;
        };

        self.scene_representation.apply_extract(extract);

        let live_probe_ids = extract
            .probes
            .iter()
            .map(|probe| probe.probe_id)
            .collect::<BTreeSet<_>>();
        let stale_resident_probe_ids = self
            .resident_slots
            .keys()
            .copied()
            .filter(|probe_id| !live_probe_ids.contains(probe_id))
            .collect::<Vec<_>>();
        for probe_id in stale_resident_probe_ids {
            self.evict_one([probe_id]);
        }
        self.pending_probes
            .retain(|probe_id| live_probe_ids.contains(probe_id));
        self.pending_updates
            .retain(|update| live_probe_ids.contains(&update.probe_id));
        self.current_requested_probe_ids
            .retain(|probe_id| live_probe_ids.contains(probe_id));
        self.probe_parent_probes
            .retain(|probe_id, parent_probe_id| {
                live_probe_ids.contains(probe_id) && live_probe_ids.contains(parent_probe_id)
            });
        self.probe_ray_budgets
            .retain(|probe_id, _| live_probe_ids.contains(probe_id));
        self.probe_scene_data
            .retain(|probe_id, _| live_probe_ids.contains(probe_id));
        self.probe_irradiance_rgb
            .retain(|probe_id, _| live_probe_ids.contains(probe_id));
        self.probe_rt_lighting_rgb
            .retain(|probe_id, _| live_probe_ids.contains(probe_id));
        self.recent_lineage_trace_support_q8
            .retain(|probe_id, _| live_probe_ids.contains(probe_id));
        self.recent_requested_lineage_support_q8
            .retain(|probe_id, _| live_probe_ids.contains(probe_id));
        let live_trace_region_ids = extract
            .trace_regions
            .iter()
            .map(|region| region.region_id)
            .collect::<BTreeSet<_>>();
        self.trace_region_scene_data
            .retain(|region_id, _| live_trace_region_ids.contains(region_id));

        self.probe_budget = (extract.probe_budget as usize)
            .max(extract.probes.iter().filter(|probe| probe.resident).count());

        for probe in &extract.probes {
            if let Some(parent_probe_id) = probe.parent_probe_id {
                self.probe_parent_probes
                    .insert(probe.probe_id, parent_probe_id);
            } else {
                self.probe_parent_probes.remove(&probe.probe_id);
            }
            self.probe_ray_budgets
                .insert(probe.probe_id, probe.ray_budget);
            self.probe_scene_data.insert(
                probe.probe_id,
                super::declarations::HybridGiRuntimeProbeSceneData {
                    position_x_q: quantized_signed(probe.position.x),
                    position_y_q: quantized_signed(probe.position.y),
                    position_z_q: quantized_signed(probe.position.z),
                    radius_q: quantized_positive(probe.radius, POSITIVE_RADIUS_SCALE),
                },
            );
            if probe.resident {
                self.promote_to_resident(probe.probe_id);
            }
        }

        for region in &extract.trace_regions {
            self.trace_region_scene_data.insert(
                region.region_id,
                super::declarations::HybridGiRuntimeTraceRegionSceneData {
                    center_x_q: quantized_signed(region.bounds_center.x),
                    center_y_q: quantized_signed(region.bounds_center.y),
                    center_z_q: quantized_signed(region.bounds_center.z),
                    radius_q: quantized_positive(region.bounds_radius, POSITIVE_RADIUS_SCALE),
                    coverage_q: quantized_positive(region.screen_coverage, POSITIVE_COVERAGE_SCALE),
                    rt_lighting_rgb: region.rt_lighting_rgb,
                },
            );
        }
    }
}

fn quantized_signed(value: f32) -> u32 {
    ((value * SIGNED_POSITION_SCALE).round() as i32).wrapping_add(SIGNED_POSITION_BIAS) as u32
}

fn quantized_positive(value: f32, scale: f32) -> u32 {
    (value.max(0.0) * scale).round() as u32
}
