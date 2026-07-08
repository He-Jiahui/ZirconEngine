use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, MutexGuard};

use zircon_runtime::core::framework::render::{
    RenderHybridGiPreparedFrame, RenderMeshSnapshot, RenderPluginRendererOutputs,
};
use zircon_runtime::core::math::{Vec3, Vec4};
use zircon_runtime::core::resource::ResourceId;
use zircon_runtime::graphics::{
    GraphicsError, RuntimePrepareCollector, RuntimePrepareCollectorContext,
    RuntimePrepareMaterialCaptureSeed,
};

use crate::hybrid_gi::renderer::{
    HybridGiGpuPendingReadback, HybridGiGpuResources, HybridGiMaterialCaptureSeed,
    HybridGiMaterialCaptureSource,
};
use crate::hybrid_gi::types::{
    HybridGiPrepareFrame, HybridGiPrepareProbe, HybridGiPrepareUpdateRequest,
    HybridGiResolveProbeSceneData, HybridGiResolveRuntime, HybridGiResolveTraceRegionSceneData,
};

use super::hybrid_gi_plugin_renderer_outputs::plugin_renderer_outputs_from_gpu_readback;

#[derive(Default)]
pub(crate) struct HybridGiRuntimePrepareCollector {
    state: Mutex<HybridGiRuntimePrepareCollectorState>,
}

#[derive(Default)]
struct HybridGiRuntimePrepareCollectorState {
    gpu_resources: Option<HybridGiGpuResources>,
    pending_readback: Option<HybridGiGpuPendingReadback>,
}

pub(crate) fn runtime_prepare_collector() -> Arc<dyn RuntimePrepareCollector> {
    Arc::new(HybridGiRuntimePrepareCollector::default())
}

impl RuntimePrepareCollector for HybridGiRuntimePrepareCollector {
    fn collect(
        &self,
        context: &mut RuntimePrepareCollectorContext<'_>,
    ) -> Result<RenderPluginRendererOutputs, GraphicsError> {
        let prepared_frame = context
            .prepared_runtime_sidebands()
            .hybrid_gi_prepared_frame()
            .cloned();
        let extract = context
            .frame_extract()
            .lighting
            .hybrid_global_illumination
            .clone();
        let scene_meshes = context.scene_snapshot().scene.meshes.clone();
        let directional_lights = context.frame_extract().lighting.directional_lights.clone();
        let point_lights = context.frame_extract().lighting.point_lights.clone();
        let spot_lights = context.frame_extract().lighting.spot_lights.clone();
        let material_capture =
            RuntimePrepareMaterialCaptureCache::from_context(context, &scene_meshes);

        let mut state = self.lock_state()?;
        let completed_readback = state
            .pending_readback
            .take()
            .map(|readback| readback.collect(context.device))
            .transpose()?;
        let outputs = plugin_renderer_outputs_from_gpu_readback(completed_readback);

        if let Some(prepared_frame) = prepared_frame.filter(|frame| !frame.is_empty()) {
            let prepare = prepare_frame_from_neutral(&prepared_frame);
            let resolve_runtime = resolve_runtime_from_neutral(&prepared_frame);
            let probe_budget = extract.as_ref().map(|extract| extract.probe_budget);
            let tracing_budget = extract
                .as_ref()
                .map(|extract| extract.tracing_budget.max(extract.trace_budget));
            let gpu_resources = state
                .gpu_resources
                .get_or_insert_with(|| HybridGiGpuResources::new(context.device));
            state.pending_readback = gpu_resources.execute_prepare(
                context.device,
                context.queue,
                &mut *context.encoder,
                &material_capture,
                Some(&prepare),
                None,
                Some(&resolve_runtime),
                extract.as_ref(),
                &scene_meshes,
                &directional_lights,
                &point_lights,
                &spot_lights,
                probe_budget,
                tracing_budget,
            )?;
        }

        Ok(outputs)
    }
}

impl HybridGiRuntimePrepareCollector {
    fn lock_state(
        &self,
    ) -> Result<MutexGuard<'_, HybridGiRuntimePrepareCollectorState>, GraphicsError> {
        self.state.lock().map_err(|_| {
            GraphicsError::AdvancedProviderSelection(
                "hybrid GI runtime prepare collector state lock poisoned".to_string(),
            )
        })
    }
}

#[derive(Default)]
struct RuntimePrepareMaterialCaptureCache {
    seeds: Vec<(ResourceId, HybridGiMaterialCaptureSeed)>,
    texture_samples: Vec<(ResourceId, Vec4)>,
}

impl RuntimePrepareMaterialCaptureCache {
    fn from_context(
        context: &RuntimePrepareCollectorContext<'_>,
        scene_meshes: &[RenderMeshSnapshot],
    ) -> Self {
        let mut cache = Self::default();
        for mesh in scene_meshes {
            let material_id = mesh.material.id();
            if cache.seeds.iter().any(|(id, _)| *id == material_id) {
                continue;
            }
            let Some(seed) = context.material_capture_seed(&material_id) else {
                continue;
            };
            cache.cache_texture_samples(context, &seed);
            cache.seeds.push((
                material_id,
                hybrid_gi_material_capture_seed_from_runtime(seed),
            ));
        }
        cache
    }

    fn cache_texture_samples(
        &mut self,
        context: &RuntimePrepareCollectorContext<'_>,
        seed: &RuntimePrepareMaterialCaptureSeed,
    ) {
        for texture_id in [
            seed.base_color_texture,
            seed.normal_texture,
            seed.metallic_roughness_texture,
            seed.occlusion_texture,
            seed.emissive_texture,
        ]
        .into_iter()
        .flatten()
        {
            if self.texture_samples.iter().any(|(id, _)| *id == texture_id) {
                continue;
            }
            if let Some(sample) = context.sample_texture_rgba(Some(texture_id), [0.5, 0.5]) {
                self.texture_samples.push((texture_id, sample));
            }
        }
    }
}

impl HybridGiMaterialCaptureSource for RuntimePrepareMaterialCaptureCache {
    fn material_capture_seed(&self, id: &ResourceId) -> Option<HybridGiMaterialCaptureSeed> {
        self.seeds
            .iter()
            .find_map(|(seed_id, seed)| (*seed_id == *id).then_some(*seed))
    }

    fn sample_texture_rgba(&self, id: Option<ResourceId>, _uv: [f32; 2]) -> Option<Vec4> {
        let id = id?;
        self.texture_samples
            .iter()
            .find_map(|(texture_id, sample)| (*texture_id == id).then_some(*sample))
    }
}

fn hybrid_gi_material_capture_seed_from_runtime(
    seed: RuntimePrepareMaterialCaptureSeed,
) -> HybridGiMaterialCaptureSeed {
    HybridGiMaterialCaptureSeed {
        base_color: seed.base_color,
        emissive: seed.emissive,
        metallic: seed.metallic,
        roughness: seed.roughness,
        double_sided: seed.double_sided,
        alpha_blend: seed.alpha_blend,
        alpha_cutoff: seed.alpha_cutoff,
        base_color_texture: seed.base_color_texture,
        normal_texture: seed.normal_texture,
        metallic_roughness_texture: seed.metallic_roughness_texture,
        occlusion_texture: seed.occlusion_texture,
        emissive_texture: seed.emissive_texture,
    }
}

fn prepare_frame_from_neutral(frame: &RenderHybridGiPreparedFrame) -> HybridGiPrepareFrame {
    HybridGiPrepareFrame {
        resident_probes: frame
            .resident_probes
            .iter()
            .map(|probe| HybridGiPrepareProbe {
                probe_id: probe.probe_id,
                slot: probe.slot,
                ray_budget: probe.ray_budget,
                irradiance_rgb: probe.irradiance_rgb,
            })
            .collect(),
        pending_updates: frame
            .pending_updates
            .iter()
            .map(|update| HybridGiPrepareUpdateRequest {
                probe_id: update.probe_id,
                ray_budget: update.ray_budget,
                generation: update.generation,
            })
            .collect(),
        scheduled_trace_region_ids: frame.scheduled_trace_region_ids.clone(),
        evictable_probe_ids: frame.evictable_probe_ids.clone(),
    }
}

fn resolve_runtime_from_neutral(frame: &RenderHybridGiPreparedFrame) -> HybridGiResolveRuntime {
    let probe_scene_data = frame
        .probe_scene_data
        .iter()
        .map(|probe| {
            (
                probe.probe_id,
                HybridGiResolveProbeSceneData::new(
                    probe.position_x_q,
                    probe.position_y_q,
                    probe.position_z_q,
                    probe.radius_q,
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let trace_region_scene_data = frame
        .trace_region_scene_data
        .iter()
        .map(|region| {
            (
                region.region_id,
                HybridGiResolveTraceRegionSceneData::new(
                    region.center_x_q,
                    region.center_y_q,
                    region.center_z_q,
                    region.radius_q,
                    region.coverage_q,
                    region.rt_lighting_rgb,
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let probe_rt_lighting_rgb = frame
        .probe_rt_lighting_rgb
        .iter()
        .map(|probe| (probe.probe_id, probe.rt_lighting_rgb))
        .collect::<BTreeMap<_, _>>();

    HybridGiResolveRuntime::new(
        probe_scene_data,
        trace_region_scene_data,
        BTreeMap::new(),
        probe_rt_lighting_rgb,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeSet::new(),
        BTreeSet::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
    )
}

#[cfg(test)]
mod tests {
    use zircon_runtime::core::framework::render::{
        RenderHybridGiPreparedProbe, RenderHybridGiPreparedProbeRtLighting,
        RenderHybridGiPreparedProbeSceneData, RenderHybridGiPreparedUpdateRequest,
    };

    use super::*;

    #[test]
    fn neutral_prepared_frame_projects_to_gpu_prepare_inputs() {
        let frame = RenderHybridGiPreparedFrame {
            resident_probes: vec![RenderHybridGiPreparedProbe {
                probe_id: 7,
                slot: 2,
                ray_budget: 32,
                irradiance_rgb: [3, 4, 5],
            }],
            pending_updates: vec![RenderHybridGiPreparedUpdateRequest {
                probe_id: 9,
                ray_budget: 64,
                generation: 11,
            }],
            scheduled_trace_region_ids: vec![44],
            evictable_probe_ids: vec![6],
            probe_scene_data: vec![RenderHybridGiPreparedProbeSceneData {
                probe_id: 7,
                position_x_q: 2000,
                position_y_q: 2010,
                position_z_q: 2020,
                radius_q: 96,
            }],
            probe_rt_lighting_rgb: vec![RenderHybridGiPreparedProbeRtLighting {
                probe_id: 7,
                rt_lighting_rgb: [64, 32, 16],
            }],
            trace_region_scene_data: Vec::new(),
        };

        let prepare = prepare_frame_from_neutral(&frame);
        let runtime = resolve_runtime_from_neutral(&frame);

        assert_eq!(prepare.resident_probes[0].probe_id, 7);
        assert_eq!(prepare.pending_updates[0].generation, 11);
        assert_eq!(prepare.scheduled_trace_region_ids, vec![44]);
        assert_eq!(prepare.evictable_probe_ids, vec![6]);
        assert_eq!(runtime.probe_scene_data(7).unwrap().position_x_q(), 2000);
        assert_eq!(runtime.probe_rt_lighting_rgb(7), Some([64, 32, 16]));
    }

    #[test]
    fn material_capture_cache_returns_seed_and_center_texture_sample() {
        let material_id = ResourceId::from_stable_label("res://materials/cache.mat");
        let texture_id = ResourceId::from_stable_label("res://textures/cache.png");
        let mut cache = RuntimePrepareMaterialCaptureCache::default();
        cache.seeds.push((
            material_id,
            HybridGiMaterialCaptureSeed {
                base_color: Vec4::ONE,
                emissive: Vec3::ZERO,
                metallic: 0.0,
                roughness: 1.0,
                double_sided: false,
                alpha_blend: false,
                alpha_cutoff: None,
                base_color_texture: Some(texture_id),
                normal_texture: None,
                metallic_roughness_texture: None,
                occlusion_texture: None,
                emissive_texture: None,
            },
        ));
        cache
            .texture_samples
            .push((texture_id, Vec4::new(0.25, 0.5, 0.75, 1.0)));

        assert_eq!(
            cache
                .material_capture_seed(&material_id)
                .unwrap()
                .base_color_texture,
            Some(texture_id)
        );
        assert_eq!(
            cache.sample_texture_rgba(Some(texture_id), [0.25, 0.75]),
            Some(Vec4::new(0.25, 0.5, 0.75, 1.0))
        );
    }
}
