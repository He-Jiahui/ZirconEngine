use super::HybridGiRuntimeState;
use zircon_runtime::core::framework::render::{
    LightmapConsumeContract, RenderDirectionalLightSnapshot, RenderHybridGiCompositePolicy,
    RenderHybridGiExtract, RenderHybridGiResolvedSettings, RenderMeshSnapshot,
    RenderPointLightSnapshot, RenderSpotLightSnapshot,
};
use zircon_runtime::core::math::Vec3;

impl HybridGiRuntimeState {
    pub(crate) fn composite_policy(&self) -> RenderHybridGiCompositePolicy {
        self.scene_representation().composite_policy()
    }

    pub(crate) fn resolved_settings(&self) -> RenderHybridGiResolvedSettings {
        self.scene_representation().resolved_settings()
    }

    pub(crate) fn register_scene_extract(
        &mut self,
        extract: Option<&RenderHybridGiExtract>,
        meshes: &[RenderMeshSnapshot],
        directional_lights: &[RenderDirectionalLightSnapshot],
        point_lights: &[RenderPointLightSnapshot],
        spot_lights: &[RenderSpotLightSnapshot],
        baked_lighting: Option<&LightmapConsumeContract>,
        has_baked_probe_grid: bool,
    ) {
        self.register_scene_extract_with_view_state(
            extract,
            meshes,
            directional_lights,
            point_lights,
            spot_lights,
            baked_lighting,
            has_baked_probe_grid,
            None,
            false,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn register_scene_extract_with_view_state(
        &mut self,
        extract: Option<&RenderHybridGiExtract>,
        meshes: &[RenderMeshSnapshot],
        directional_lights: &[RenderDirectionalLightSnapshot],
        point_lights: &[RenderPointLightSnapshot],
        spot_lights: &[RenderSpotLightSnapshot],
        baked_lighting: Option<&LightmapConsumeContract>,
        has_baked_probe_grid: bool,
        camera_position: Option<Vec3>,
        history_invalidated: bool,
    ) {
        let enabled_extract = extract.filter(|extract| extract.enabled);
        self.register_extract(enabled_extract);
        if enabled_extract.is_some() {
            self.scene_representation_mut()
                .synchronize_scene_with_baked_and_view_state(
                    meshes,
                    directional_lights,
                    point_lights,
                    spot_lights,
                    baked_lighting,
                    has_baked_probe_grid,
                    camera_position,
                    history_invalidated,
                );
        }
    }

    pub(crate) fn register_extract(&mut self, extract: Option<&RenderHybridGiExtract>) {
        self.clear_evictable_probes();
        self.clear_scheduled_trace_regions();
        self.current_requested_probe_ids_mut().clear();

        let Some(extract) = extract.filter(|extract| extract.enabled) else {
            *self = Self::default();
            return;
        };

        self.scene_representation_mut().apply_extract(extract);

        let stale_resident_probe_ids = self.resident_probe_ids().collect::<Vec<_>>();
        for probe_id in stale_resident_probe_ids {
            self.evict_one([probe_id]);
        }
        self.retain_pending_probes(|_| false);
        self.retain_pending_update_requests(|_| false);
        self.current_requested_probe_ids_mut().clear();
        self.probe_parent_probes_mut().clear();
        self.probe_ray_budgets_mut().clear();
        self.probe_scene_data_mut().clear();
        self.probe_irradiance_rgb_mut().clear();
        self.probe_rt_lighting_rgb_mut().clear();
        self.recent_lineage_trace_support_q8_mut().clear();
        self.recent_requested_lineage_support_q8_mut().clear();
        self.trace_region_scene_data_mut().clear();
        self.set_probe_budget(0);

        self.rebuild_probe_child_probes();
    }
}
