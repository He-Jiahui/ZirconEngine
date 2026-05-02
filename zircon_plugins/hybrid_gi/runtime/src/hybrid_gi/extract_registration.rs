use std::collections::BTreeSet;

use zircon_runtime::core::framework::render::{
    RenderDirectionalLightSnapshot, RenderHybridGiExtract, RenderMeshSnapshot,
    RenderPointLightSnapshot, RenderSpotLightSnapshot,
};
use zircon_runtime::graphics::hybrid_gi_extract_sources::{
    enabled_hybrid_gi_extract, hybrid_gi_extract_uses_scene_representation_budget,
};

use super::{
    first_hybrid_gi_runtime_probe_payloads, first_hybrid_gi_runtime_trace_region_payloads,
    HybridGiRuntimeState,
};

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
        let enabled_extract = enabled_hybrid_gi_extract(extract);
        self.register_extract(enabled_extract);
        if enabled_extract.is_some() {
            self.scene_representation_mut().synchronize_scene(
                meshes,
                directional_lights,
                point_lights,
                spot_lights,
            );
        }
    }

    pub(crate) fn register_extract(&mut self, extract: Option<&RenderHybridGiExtract>) {
        self.clear_evictable_probes();
        self.clear_scheduled_trace_regions();
        self.current_requested_probe_ids_mut().clear();

        let Some(extract) = enabled_hybrid_gi_extract(extract) else {
            *self = Self::default();
            return;
        };

        self.scene_representation_mut().apply_extract(extract);

        let scene_representation_owns_runtime =
            hybrid_gi_extract_uses_scene_representation_budget(extract);
        let extract_payloads_drive_runtime = !scene_representation_owns_runtime;
        let live_probe_payloads = if extract_payloads_drive_runtime {
            first_hybrid_gi_runtime_probe_payloads(extract)
        } else {
            Vec::new()
        };
        let scene_probe_payloads = if extract_payloads_drive_runtime {
            live_probe_payloads.clone()
        } else {
            Vec::new()
        };
        let live_probe_ids = live_probe_payloads
            .iter()
            .map(|probe| probe.probe_id)
            .collect::<BTreeSet<_>>();
        let scene_probe_ids = scene_probe_payloads
            .iter()
            .map(|probe| probe.probe_id)
            .collect::<BTreeSet<_>>();
        let stale_resident_probe_ids = self
            .resident_probe_ids()
            .filter(|probe_id| !live_probe_ids.contains(probe_id))
            .collect::<Vec<_>>();
        for probe_id in stale_resident_probe_ids {
            self.evict_one([probe_id]);
        }
        self.retain_pending_probes(|probe_id| live_probe_ids.contains(probe_id));
        self.retain_pending_update_requests(|update| live_probe_ids.contains(&update.probe_id()));
        self.current_requested_probe_ids_mut()
            .retain(|probe_id| live_probe_ids.contains(probe_id));
        self.probe_parent_probes_mut()
            .retain(|probe_id, parent_probe_id| {
                live_probe_ids.contains(probe_id) && live_probe_ids.contains(parent_probe_id)
            });
        self.probe_ray_budgets_mut()
            .retain(|probe_id, _| scene_probe_ids.contains(probe_id));
        self.probe_scene_data_mut()
            .retain(|probe_id, _| scene_probe_ids.contains(probe_id));
        self.probe_irradiance_rgb_mut()
            .retain(|probe_id, _| live_probe_ids.contains(probe_id));
        self.probe_rt_lighting_rgb_mut()
            .retain(|probe_id, _| live_probe_ids.contains(probe_id));
        self.recent_lineage_trace_support_q8_mut()
            .retain(|probe_id, _| live_probe_ids.contains(probe_id));
        self.recent_requested_lineage_support_q8_mut()
            .retain(|probe_id, _| live_probe_ids.contains(probe_id));
        let live_trace_region_payloads = if extract_payloads_drive_runtime {
            first_hybrid_gi_runtime_trace_region_payloads(extract)
        } else {
            Vec::new()
        };
        let live_trace_region_ids = live_trace_region_payloads
            .iter()
            .map(|region| region.region_id)
            .collect::<BTreeSet<_>>();
        self.trace_region_scene_data_mut()
            .retain(|region_id, _| live_trace_region_ids.contains(region_id));

        let unique_resident_probe_count = live_probe_payloads
            .iter()
            .filter(|probe| probe.resident)
            .count();
        self.set_probe_budget(if extract_payloads_drive_runtime {
            (extract.probe_budget as usize).max(unique_resident_probe_count)
        } else {
            0
        });

        for probe in &scene_probe_payloads {
            self.probe_ray_budgets_mut()
                .insert(probe.probe_id, probe.ray_budget);
            self.probe_scene_data_mut().insert(
                probe.probe_id,
                super::declarations::HybridGiRuntimeProbeSceneData::new(
                    quantized_signed(probe.position.x),
                    quantized_signed(probe.position.y),
                    quantized_signed(probe.position.z),
                    quantized_positive(probe.radius, POSITIVE_RADIUS_SCALE),
                ),
            );
        }

        for probe in &live_probe_payloads {
            if let Some(parent_probe_id) = probe
                .parent_probe_id
                .filter(|parent_probe_id| live_probe_ids.contains(parent_probe_id))
            {
                self.probe_parent_probes_mut()
                    .insert(probe.probe_id, parent_probe_id);
            } else {
                self.probe_parent_probes_mut().remove(&probe.probe_id);
            }
            if probe.resident {
                self.promote_to_resident(probe.probe_id);
            }
        }

        for region in &live_trace_region_payloads {
            self.trace_region_scene_data_mut().insert(
                region.region_id,
                super::declarations::HybridGiRuntimeTraceRegionSceneData::new(
                    quantized_signed(region.bounds_center.x),
                    quantized_signed(region.bounds_center.y),
                    quantized_signed(region.bounds_center.z),
                    quantized_positive(region.bounds_radius, POSITIVE_RADIUS_SCALE),
                    quantized_positive(region.screen_coverage, POSITIVE_COVERAGE_SCALE),
                    region.rt_lighting_rgb,
                ),
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
