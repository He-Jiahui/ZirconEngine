use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use zircon_runtime::core::framework::render::{
    RenderBudgetKey, RenderHybridGiPreparedFrame, RenderHybridGiQuality, RenderMeshSnapshot,
    RenderPluginRendererOutputs,
};
use zircon_runtime::core::math::Vec3;
use zircon_runtime::graphics::{
    GraphicsError, RuntimePrepareCollector, RuntimePrepareCollectorContext,
};

use crate::hybrid_gi::renderer::{
    GlobalSdfGpuBuildStats, GlobalSdfGpuReadbackFuture, GlobalSdfGpuState,
    HybridGiGpuReadbackFuture, HybridGiGpuResources, HybridGiMaterialCaptureSeed,
    HybridGiMaterialCaptureSource, RadianceCacheGpuState,
};
use crate::hybrid_gi::scene_representation::{
    HybridGiGlobalSdfClipmapBounds, HybridGiGlobalSdfSceneState, HybridGiMeshSdfAssetState,
    HybridGiMeshSdfMaterialFlags, HybridGiMeshSdfObject, HybridGiMeshSdfSceneState,
};

use super::hybrid_gi_plugin_renderer_outputs::plugin_renderer_outputs_from_gpu_readback;

mod global_sdf_stats;
mod material_capture;
mod mesh_projection;
mod neutral_projection;

use global_sdf_stats::{global_sdf_runtime_stats, GlobalSdfCpuPrepareTimings};
use material_capture::RuntimePrepareMaterialCaptureCache;
use mesh_projection::RuntimePrepareMeshProjectionCache;
use neutral_projection::{
    prepare_frame_from_neutral, radiance_cache_consumes_from_neutral,
    radiance_cache_updates_for_instance, resolve_runtime_from_neutral, scene_prepare_from_neutral,
};

const MAX_RADIANCE_CACHE_GPU_INSTANCE_COUNT: usize = 32;
const GLOBAL_SDF_LOW_PAGE_BUDGET: usize = 32;
const GLOBAL_SDF_MEDIUM_PAGE_BUDGET: usize = 64;
const GLOBAL_SDF_HIGH_PAGE_BUDGET: usize = 128;
const GLOBAL_SDF_LOW_BUILD_PAGE_BUDGET: usize = 8;
const GLOBAL_SDF_MEDIUM_BUILD_PAGE_BUDGET: usize = 16;
const GLOBAL_SDF_HIGH_BUILD_PAGE_BUDGET: usize = 32;
const HGI_RUNTIME_PREPARE_EXECUTOR_ID: &str = "hybrid-gi.runtime-prepare";
const GLOBAL_SDF_BUILD_PROFILE_NAME: &str = "runtime_prepare.hybrid_gi.global_sdf_build";
const HGI_PREPARE_PROFILE_NAME: &str = "runtime_prepare.hybrid_gi.prepare";

#[derive(Default)]
pub(crate) struct HybridGiRuntimePrepareCollector {
    state: Mutex<HybridGiRuntimePrepareCollectorState>,
}

#[derive(Default)]
struct HybridGiRuntimePrepareCollectorState {
    gpu_resources: Option<HybridGiGpuResources>,
    radiance_cache_instances: BTreeMap<u64, HybridGiRuntimePrepareInstanceState>,
    collector_frame_index: u64,
}

struct HybridGiRuntimePrepareInstanceState {
    radiance_cache_gpu_state: RadianceCacheGpuState,
    global_sdf_gpu_state: GlobalSdfGpuState,
    bootstrap: RadianceCacheBootstrapState,
    mesh_sdf_scene_state: HybridGiMeshSdfSceneState,
    mesh_projection_cache: RuntimePrepareMeshProjectionCache,
    global_sdf_scene_state: HybridGiGlobalSdfSceneState,
    pending_readbacks: VecDeque<HybridGiRuntimePreparePendingReadback>,
    pending_global_sdf_readbacks: VecDeque<GlobalSdfGpuReadbackFuture>,
    global_sdf_deferred_cursor: usize,
    last_used_frame: u64,
}

struct HybridGiRuntimePreparePendingReadback {
    radiance_cache_revision: u64,
    future: HybridGiGpuReadbackFuture,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RadianceCacheSubmission {
    revision: u64,
    uses_bootstrap_snapshot: bool,
}

#[derive(Debug)]
struct RadianceCacheBootstrapState {
    required_revision: u64,
    confirmed_revision: u64,
}

impl Default for RadianceCacheBootstrapState {
    fn default() -> Self {
        Self {
            required_revision: 1,
            confirmed_revision: 0,
        }
    }
}

impl RadianceCacheBootstrapState {
    fn begin_submission(&mut self, has_incremental_updates: bool) -> RadianceCacheSubmission {
        let uses_bootstrap_snapshot = self.confirmed_revision < self.required_revision;
        if has_incremental_updates {
            self.required_revision = self.required_revision.saturating_add(1);
        }
        RadianceCacheSubmission {
            revision: self.required_revision,
            uses_bootstrap_snapshot,
        }
    }

    fn confirm_submission(&mut self, revision: u64) {
        self.confirmed_revision = self
            .confirmed_revision
            .max(revision.min(self.required_revision));
    }
}

pub(crate) fn runtime_prepare_collector() -> Arc<dyn RuntimePrepareCollector> {
    Arc::new(HybridGiRuntimePrepareCollector::default())
}

impl RuntimePrepareCollector for HybridGiRuntimePrepareCollector {
    fn collect(
        &self,
        context: &mut RuntimePrepareCollectorContext<'_>,
    ) -> Result<RenderPluginRendererOutputs, GraphicsError> {
        let Some(prepared_frame) = context
            .prepared_runtime_sidebands()
            .hybrid_gi_prepared_frame()
            .cloned()
            .filter(|frame| !frame.is_empty())
        else {
            return Ok(RenderPluginRendererOutputs::default());
        };
        let Some(extract) = context
            .frame_extract()
            .lighting
            .hybrid_global_illumination
            .clone()
            .filter(|extract| extract.enabled)
        else {
            return Ok(RenderPluginRendererOutputs::default());
        };
        let scene_meshes = context.scene_meshes();
        let directional_lights = context.frame_extract().lighting.directional_lights.clone();
        let point_lights = context.frame_extract().lighting.point_lights.clone();
        let spot_lights = context.frame_extract().lighting.spot_lights.clone();
        let camera_position = context
            .frame_extract()
            .view
            .selected_effective_camera()
            .transform
            .translation;
        let global_sdf_clipmap_bounds =
            HybridGiGlobalSdfSceneState::clipmap_bounds_for_camera(camera_position);
        let global_sdf_page_budget = global_sdf_page_budget(extract.quality);
        let global_sdf_build_page_budget = global_sdf_build_page_budget(extract.quality);

        let mut state = self.lock_state()?;
        let instance_id = prepared_frame.radiance_cache_instance_id;
        if instance_id == 0 {
            return Err(GraphicsError::AdvancedProviderSelection(
                "hybrid GI prepared frame is missing its radiance-cache instance identity"
                    .to_string(),
            ));
        }
        state.collector_frame_index = state.collector_frame_index.saturating_add(1).max(1);
        let collector_frame_index = state.collector_frame_index;
        state.ensure_instance(instance_id, collector_frame_index, context.device);

        let prepare = prepare_frame_from_neutral(&prepared_frame);
        let resolve_runtime = resolve_runtime_from_neutral(&prepared_frame);
        let radiance_cache_consumes = radiance_cache_consumes_from_neutral(&prepared_frame);
        let probe_budget = Some(
            (prepared_frame.resident_probes.len() + prepared_frame.pending_updates.len()) as u32,
        );
        let tracing_budget = Some(match extract.quality {
            RenderHybridGiQuality::Low => 8,
            RenderHybridGiQuality::Medium => 16,
            RenderHybridGiQuality::High => 32,
        });
        let HybridGiRuntimePrepareCollectorState {
            gpu_resources,
            radiance_cache_instances,
            ..
        } = &mut *state;
        let Some(instance) = radiance_cache_instances.get_mut(&instance_id) else {
            return Err(GraphicsError::AdvancedProviderSelection(format!(
                "hybrid GI runtime-prepare instance {instance_id} was not retained after insertion"
            )));
        };
        let global_sdf_completions = instance.collect_ready_global_sdf_builds()?;
        let global_sdf_uploaded_page_count = global_sdf_completions.len();
        instance
            .global_sdf_scene_state
            .commit_pages(&global_sdf_completions);
        let completed_readback = instance.collect_ready_readbacks()?;
        if !context.gpu_work_admitted() {
            return Ok(plugin_renderer_outputs_from_gpu_readback(
                completed_readback,
                None,
            ));
        }
        let gpu_resources =
            gpu_resources.get_or_insert_with(|| HybridGiGpuResources::new(context.device));
        let mesh_projection_started = Instant::now();
        let mesh_projection_cache_hit = instance
            .mesh_projection_cache
            .can_reuse(scene_meshes, &global_sdf_clipmap_bounds);
        let (mesh_sdf_sync, mesh_object_collection_time_us, mesh_scene_sync_time_us) =
            if mesh_projection_cache_hit {
                (
                    Default::default(),
                    elapsed_micros(mesh_projection_started),
                    0,
                )
            } else {
                instance
                    .mesh_projection_cache
                    .refresh_material_capture(context, scene_meshes);
                let mesh_projection = mesh_projection_from_context(
                    context,
                    instance.mesh_projection_cache.material_capture(),
                    scene_meshes,
                    &global_sdf_clipmap_bounds,
                );
                let mesh_scene_sync_started = Instant::now();
                let sync = instance
                    .mesh_sdf_scene_state
                    .synchronize(mesh_projection.mesh_sdf_objects);
                let mesh_scene_sync_time_us = elapsed_micros(mesh_scene_sync_started);
                instance.mesh_projection_cache.capture(
                    scene_meshes,
                    &global_sdf_clipmap_bounds,
                    mesh_projection.scene_mesh_world_bounds,
                );
                let mesh_object_collection_time_us =
                    elapsed_micros(mesh_projection_started).saturating_sub(mesh_scene_sync_time_us);
                (
                    sync,
                    mesh_object_collection_time_us,
                    mesh_scene_sync_time_us,
                )
            };
        let global_sdf_residency_started = Instant::now();
        let global_sdf_residency_changed = instance.global_sdf_scene_state.synchronize(
            camera_position,
            mesh_sdf_sync.dirty_regions(),
            global_sdf_page_budget,
        );
        let global_sdf_residency_time_us = elapsed_micros(global_sdf_residency_started);
        let global_sdf_influence_update_started = Instant::now();
        if global_sdf_residency_changed || !mesh_sdf_sync.dirty_regions().is_empty() {
            instance
                .global_sdf_scene_state
                .synchronize_influence(instance.mesh_sdf_scene_state.objects());
        }
        let global_sdf_influence_update_time_us =
            elapsed_micros(global_sdf_influence_update_started);
        let global_sdf_candidate_build_started = Instant::now();
        let mut global_sdf_build_requests = instance
            .global_sdf_scene_state
            .dirty_page_build_requests()
            .into_iter()
            .filter(|request| !instance.has_in_flight_global_sdf_request(*request))
            .collect::<Vec<_>>();
        let mut global_sdf_build_stats = GlobalSdfGpuBuildStats::default();
        if !global_sdf_build_requests.is_empty() {
            if can_enqueue_readback_observation(instance.in_flight_readback_count()) {
                rotate_global_sdf_build_requests(
                    &mut global_sdf_build_requests,
                    &mut instance.global_sdf_deferred_cursor,
                    global_sdf_build_page_budget,
                );
                let gpu_pass = context.begin_gpu_pass(GLOBAL_SDF_BUILD_PROFILE_NAME);
                let gpu_dispatch_started = Instant::now();
                let dispatch = gpu_resources.dispatch_global_sdf_pages(
                    &instance.global_sdf_gpu_state,
                    context.device,
                    &mut *context.encoder,
                    &mut instance.global_sdf_scene_state,
                    instance.mesh_sdf_scene_state.objects(),
                    &global_sdf_build_requests,
                    global_sdf_build_page_budget,
                );
                if dispatch.encoded_gpu_work() {
                    context.end_gpu_pass(
                        gpu_pass,
                        HGI_RUNTIME_PREPARE_EXECUTOR_ID,
                        RenderBudgetKey::Other,
                        elapsed_micros(gpu_dispatch_started),
                    );
                } else {
                    context.discard_gpu_pass(gpu_pass);
                }
                global_sdf_build_stats = dispatch.stats();
                if let Some(pending) = dispatch.into_pending() {
                    let future = pending.enqueue(context)?;
                    instance.pending_global_sdf_readbacks.push_back(future);
                }
            } else {
                global_sdf_build_stats = GlobalSdfGpuBuildStats::deferred_by_readback_backpressure(
                    global_sdf_build_requests.len(),
                );
            }
        }
        let global_sdf_candidate_build_time_us = elapsed_micros(global_sdf_candidate_build_started);
        let scene_mesh_world_bounds = instance.mesh_projection_cache.scene_mesh_world_bounds();
        let scene_prepare =
            scene_prepare_from_neutral(&prepared_frame, scene_mesh_world_bounds.as_ref());
        let execution_scene_meshes = instance.mesh_projection_cache.scene_meshes();
        let global_sdf_stats = global_sdf_runtime_stats(
            &instance.global_sdf_scene_state,
            &instance.global_sdf_gpu_state,
            GlobalSdfCpuPrepareTimings {
                mesh_object_collection_time_us,
                mesh_scene_sync_time_us,
                global_sdf_residency_time_us,
                global_sdf_influence_update_time_us,
                global_sdf_candidate_build_time_us,
                mesh_projection_cache_hit,
            },
            instance.mesh_sdf_scene_state.objects().len(),
            global_sdf_uploaded_page_count,
            global_sdf_build_stats,
        );
        let outputs =
            plugin_renderer_outputs_from_gpu_readback(completed_readback, Some(global_sdf_stats));
        if !can_enqueue_readback_observation(instance.in_flight_readback_count()) {
            return Ok(outputs);
        }
        let cache_submission = instance
            .bootstrap
            .begin_submission(!prepared_frame.radiance_cache_updates.is_empty());
        let radiance_cache_updates = radiance_cache_updates_for_instance(
            &prepared_frame,
            cache_submission.uses_bootstrap_snapshot,
        );
        let gpu_pass = context.begin_gpu_pass(HGI_PREPARE_PROFILE_NAME);
        let gpu_prepare_started = Instant::now();
        let pending_readback = gpu_resources.execute_prepare(
            &instance.radiance_cache_gpu_state,
            &instance.global_sdf_gpu_state,
            &instance.global_sdf_scene_state,
            context.device,
            context.queue,
            &mut *context.encoder,
            instance.mesh_projection_cache.material_capture(),
            Some(&prepare),
            scene_prepare.as_ref(),
            &radiance_cache_updates,
            &radiance_cache_consumes,
            Some(&resolve_runtime),
            scene_mesh_world_bounds,
            execution_scene_meshes,
            &directional_lights,
            &point_lights,
            &spot_lights,
            probe_budget,
            tracing_budget,
        );
        context.end_gpu_pass(
            gpu_pass,
            HGI_RUNTIME_PREPARE_EXECUTOR_ID,
            RenderBudgetKey::Other,
            elapsed_micros(gpu_prepare_started),
        );
        let pending_readback = pending_readback?;
        if let Some(pending_readback) = pending_readback {
            let future = pending_readback.enqueue(context)?;
            instance
                .pending_readbacks
                .push_back(HybridGiRuntimePreparePendingReadback {
                    radiance_cache_revision: cache_submission.revision,
                    future,
                });
        }

        Ok(outputs)
    }
}

impl HybridGiRuntimePrepareCollectorState {
    fn ensure_instance(
        &mut self,
        instance_id: u64,
        collector_frame_index: u64,
        device: &wgpu::Device,
    ) {
        let instance_created = !self.radiance_cache_instances.contains_key(&instance_id);
        if instance_created
            && self.radiance_cache_instances.len() >= MAX_RADIANCE_CACHE_GPU_INSTANCE_COUNT
        {
            let evict_id = self
                .radiance_cache_instances
                .iter()
                .min_by_key(|(id, instance)| (instance.last_used_frame, **id))
                .map(|(id, _)| *id);
            if let Some(evict_id) = evict_id {
                self.radiance_cache_instances.remove(&evict_id);
            }
        }
        let instance = self
            .radiance_cache_instances
            .entry(instance_id)
            .or_insert_with(|| HybridGiRuntimePrepareInstanceState {
                radiance_cache_gpu_state: RadianceCacheGpuState::new(device),
                global_sdf_gpu_state: GlobalSdfGpuState::new(device),
                bootstrap: RadianceCacheBootstrapState::default(),
                mesh_sdf_scene_state: HybridGiMeshSdfSceneState::default(),
                mesh_projection_cache: RuntimePrepareMeshProjectionCache::default(),
                global_sdf_scene_state: HybridGiGlobalSdfSceneState::default(),
                pending_readbacks: VecDeque::new(),
                pending_global_sdf_readbacks: VecDeque::new(),
                global_sdf_deferred_cursor: 0,
                last_used_frame: collector_frame_index,
            });
        instance.last_used_frame = collector_frame_index;
    }
}

impl HybridGiRuntimePrepareInstanceState {
    fn in_flight_readback_count(&self) -> usize {
        self.pending_readbacks.len() + self.pending_global_sdf_readbacks.len()
    }

    fn has_in_flight_global_sdf_request(
        &self,
        request: crate::hybrid_gi::scene_representation::HybridGiGlobalSdfPageBuildRequest,
    ) -> bool {
        self.pending_global_sdf_readbacks
            .iter()
            .any(|pending| pending.requests().contains(&request))
    }

    fn collect_ready_global_sdf_builds(
        &mut self,
    ) -> Result<
        Vec<crate::hybrid_gi::scene_representation::HybridGiGlobalSdfPageBuildRequest>,
        GraphicsError,
    > {
        let mut completed = Vec::new();
        while self
            .pending_global_sdf_readbacks
            .front()
            .is_some_and(GlobalSdfGpuReadbackFuture::is_ready)
        {
            let Some(pending) = self.pending_global_sdf_readbacks.pop_front() else {
                break;
            };
            if let Some(result) = pending.try_collect() {
                let mut pages = result?;
                completed.append(&mut pages);
            }
        }
        Ok(completed)
    }

    fn collect_ready_readbacks(
        &mut self,
    ) -> Result<Option<crate::hybrid_gi::renderer::HybridGiGpuReadback>, GraphicsError> {
        let mut latest_success = None;
        while self
            .pending_readbacks
            .front()
            .is_some_and(|pending| pending.future.is_ready())
        {
            let Some(pending) = self.pending_readbacks.pop_front() else {
                break;
            };
            let Some(result) = pending.future.try_collect() else {
                continue;
            };
            let readback = result?;
            self.bootstrap
                .confirm_submission(pending.radiance_cache_revision);
            latest_success = Some(readback);
        }
        Ok(latest_success)
    }
}

fn can_enqueue_readback_observation(pending_readback_count: usize) -> bool {
    pending_readback_count < RuntimePrepareCollectorContext::MAX_IN_FLIGHT_GPU_READBACK_FRAMES
}

fn elapsed_micros(started: Instant) -> u64 {
    started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
}

fn global_sdf_page_budget(quality: RenderHybridGiQuality) -> usize {
    match quality {
        RenderHybridGiQuality::Low => GLOBAL_SDF_LOW_PAGE_BUDGET,
        RenderHybridGiQuality::Medium => GLOBAL_SDF_MEDIUM_PAGE_BUDGET,
        RenderHybridGiQuality::High => GLOBAL_SDF_HIGH_PAGE_BUDGET,
    }
}

fn global_sdf_build_page_budget(quality: RenderHybridGiQuality) -> usize {
    match quality {
        RenderHybridGiQuality::Low => GLOBAL_SDF_LOW_BUILD_PAGE_BUDGET,
        RenderHybridGiQuality::Medium => GLOBAL_SDF_MEDIUM_BUILD_PAGE_BUDGET,
        RenderHybridGiQuality::High => GLOBAL_SDF_HIGH_BUILD_PAGE_BUDGET,
    }
}

fn rotate_global_sdf_build_requests<T>(requests: &mut [T], cursor: &mut usize, budget: usize) {
    if requests.is_empty() {
        *cursor = 0;
        return;
    }
    let start = *cursor % requests.len();
    requests.rotate_left(start);
    *cursor = (start + budget.max(1)) % requests.len();
}

struct RuntimePrepareMeshProjection {
    mesh_sdf_objects: Vec<HybridGiMeshSdfObject>,
    scene_mesh_world_bounds: Arc<
        [(
            u64,
            zircon_runtime::core::framework::render::RenderMeshBounds,
        )],
    >,
}

fn mesh_projection_from_context(
    context: &RuntimePrepareCollectorContext<'_>,
    material_capture: &RuntimePrepareMaterialCaptureCache,
    scene_meshes: &[RenderMeshSnapshot],
    clipmaps: &[HybridGiGlobalSdfClipmapBounds],
) -> RuntimePrepareMeshProjection {
    let mut mesh_sdf_objects = Vec::with_capacity(scene_meshes.len());
    let mut scene_mesh_world_bounds = Vec::with_capacity(scene_meshes.len());
    for mesh in scene_meshes {
        let Some(geometry) = context.mesh_geometry_seed(mesh) else {
            continue;
        };
        scene_mesh_world_bounds.push((
            mesh.stable_instance_key,
            geometry.local_bounds.transformed(mesh.transform),
        ));
        let material = material_capture
            .material_capture_seed(&mesh.material.id())
            .map_or_else(HybridGiMeshSdfMaterialFlags::default, |seed| {
                HybridGiMeshSdfMaterialFlags {
                    casts_shadows: seed.cast_shadows,
                    emissive: (seed.emissive.is_finite() && seed.emissive.max_element() > 0.0)
                        || seed.emissive_texture.is_some(),
                }
            });
        mesh_sdf_objects.push(HybridGiMeshSdfObject::from_sources(
            mesh,
            geometry.local_bounds,
            geometry.resource_revision,
            geometry.shape_revision,
            HybridGiMeshSdfAssetState::from_runtime(geometry.mesh_sdf),
            material,
            clipmaps,
        ));
    }
    RuntimePrepareMeshProjection {
        mesh_sdf_objects,
        scene_mesh_world_bounds: scene_mesh_world_bounds.into(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radiance_cache_bootstrap_requires_successful_gpu_observation() {
        let mut bootstrap = RadianceCacheBootstrapState::default();

        let initial = bootstrap.begin_submission(false);
        assert!(initial.uses_bootstrap_snapshot);
        assert_eq!(initial.revision, 1);
        assert!(bootstrap.begin_submission(false).uses_bootstrap_snapshot);

        bootstrap.confirm_submission(initial.revision);
        let stable = bootstrap.begin_submission(false);
        assert!(!stable.uses_bootstrap_snapshot);
        assert_eq!(stable.revision, initial.revision);
    }

    #[test]
    fn radiance_cache_update_retries_as_bootstrap_until_latest_revision_is_observed() {
        let mut bootstrap = RadianceCacheBootstrapState::default();
        let initial = bootstrap.begin_submission(false);
        bootstrap.confirm_submission(initial.revision);

        let incremental = bootstrap.begin_submission(true);
        assert!(!incremental.uses_bootstrap_snapshot);
        assert_eq!(incremental.revision, 2);
        let retry = bootstrap.begin_submission(false);
        assert!(retry.uses_bootstrap_snapshot);
        assert_eq!(retry.revision, incremental.revision);

        bootstrap.confirm_submission(initial.revision);
        assert!(bootstrap.begin_submission(false).uses_bootstrap_snapshot);
        bootstrap.confirm_submission(incremental.revision);
        assert!(!bootstrap.begin_submission(false).uses_bootstrap_snapshot);
    }

    #[test]
    fn radiance_cache_readback_observation_is_bounded_to_the_shared_frame_ring() {
        let capacity = RuntimePrepareCollectorContext::MAX_IN_FLIGHT_GPU_READBACK_FRAMES;

        assert!(can_enqueue_readback_observation(capacity.saturating_sub(1)));
        assert!(!can_enqueue_readback_observation(capacity));
        assert!(!can_enqueue_readback_observation(
            capacity.saturating_add(1)
        ));
    }

    #[test]
    fn global_sdf_deferred_requests_rotate_across_bounded_batches() {
        let mut requests = [0_u32, 1, 2, 3, 4];
        let mut cursor = 0;

        rotate_global_sdf_build_requests(&mut requests, &mut cursor, 2);
        assert_eq!(requests, [0, 1, 2, 3, 4]);
        assert_eq!(cursor, 2);

        requests.sort_unstable();
        rotate_global_sdf_build_requests(&mut requests, &mut cursor, 2);
        assert_eq!(requests, [2, 3, 4, 0, 1]);
        assert_eq!(cursor, 4);

        requests.sort_unstable();
        rotate_global_sdf_build_requests(&mut requests, &mut cursor, 2);
        assert_eq!(requests, [4, 0, 1, 2, 3]);
        assert_eq!(cursor, 1);
    }

    #[test]
    fn global_sdf_readback_backpressure_reports_deferred_work() {
        let stats = GlobalSdfGpuBuildStats::deferred_by_readback_backpressure(7);

        assert_eq!(stats.deferred_page_count, 7);
        assert_eq!(stats.dispatched_page_count, 0);
        assert_eq!(stats.transient_upload_byte_count, 0);
    }

    #[test]
    fn new_hgi_gpu_work_requires_shared_readback_admission_before_dispatch() {
        let source = include_str!("runtime_prepare_collector.rs");
        let admission_gate = ["if !context.", "gpu_work_admitted() {"].concat();
        let admission_gate = source
            .find(&admission_gate)
            .expect("runtime prepare must reject new GPU work without a shared readback slot");
        let global_sdf_dispatch = source
            .find("gpu_resources.dispatch_global_sdf_pages(")
            .expect("global SDF dispatch must remain explicit");
        let prepare_dispatch = source
            .find("gpu_resources.execute_prepare(")
            .expect("HGI prepare dispatch must remain explicit");

        assert!(admission_gate < global_sdf_dispatch);
        assert!(admission_gate < prepare_dispatch);
    }

    #[test]
    fn hgi_gpu_dispatches_publish_named_shared_timer_scopes() {
        let source = include_str!("runtime_prepare_collector.rs");
        let global_scope = ["context.begin_gpu_pass(", "GLOBAL_SDF_BUILD_PROFILE_NAME"].concat();
        let prepare_scope = ["context.begin_gpu_pass(", "HGI_PREPARE_PROFILE_NAME"].concat();
        let end_scope = ["context.end_gpu_", "pass("].concat();
        let global_dispatch = source
            .find("gpu_resources.dispatch_global_sdf_pages(")
            .expect("Global SDF dispatch must remain explicit");
        let prepare_dispatch = source
            .find("gpu_resources.execute_prepare(")
            .expect("radiance-cache prepare dispatch must remain explicit");
        let global_scope = source
            .find(&global_scope)
            .expect("Global SDF dispatch must open a shared timer scope");
        let prepare_scope = source
            .find(&prepare_scope)
            .expect("radiance-cache prepare dispatch must open a shared timer scope");
        let global_end = source[global_dispatch..]
            .find(&end_scope)
            .map(|offset| global_dispatch + offset)
            .expect("Global SDF dispatch must close its shared timer scope");
        let prepare_end = source[prepare_dispatch..]
            .find(&end_scope)
            .map(|offset| prepare_dispatch + offset)
            .expect("radiance-cache prepare dispatch must close its shared timer scope");

        assert!(source.contains("runtime_prepare.hybrid_gi.global_sdf_build"));
        assert!(source.contains("runtime_prepare.hybrid_gi.prepare"));
        assert!(global_scope < global_dispatch);
        assert!(global_dispatch < global_end);
        assert!(prepare_scope < prepare_dispatch);
        assert!(prepare_dispatch < prepare_end);
    }

    #[test]
    fn empty_global_sdf_dispatch_closes_its_scope_without_publishing_a_profile() {
        let source = include_str!("runtime_prepare_collector.rs");
        let global_dispatch = source
            .find("gpu_resources.dispatch_global_sdf_pages(")
            .expect("Global SDF dispatch must remain explicit");
        let dispatch_profile = source[global_dispatch..]
            .find("if dispatch.encoded_gpu_work() {")
            .map(|offset| global_dispatch + offset)
            .expect("Global SDF profile must be conditional on encoded GPU work");
        let discard_scope = source[dispatch_profile..]
            .find("context.discard_gpu_pass(gpu_pass);")
            .map(|offset| dispatch_profile + offset)
            .expect("empty Global SDF dispatch must close its timer scope");
        let stats = source[global_dispatch..]
            .find("global_sdf_build_stats = dispatch.stats();")
            .map(|offset| global_dispatch + offset)
            .expect("Global SDF dispatch statistics must remain available");

        assert!(global_dispatch < dispatch_profile);
        assert!(dispatch_profile < discard_scope);
        assert!(discard_scope < stats);
    }
}
