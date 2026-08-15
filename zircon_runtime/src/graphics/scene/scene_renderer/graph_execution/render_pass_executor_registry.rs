use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::graphics::feature::COMPUTE_GENERIC_EXECUTOR_ID;
use crate::graphics::scene::anti_alias::fxaa::FXAA_EXECUTOR_ID;
use crate::graphics::scene::anti_alias::smaa::SMAA_EXECUTOR_ID;
use crate::graphics::scene::scene_renderer::environment::ibl_bake_compute_executor::ibl_bake_compute_executor_registrations;
use crate::graphics::CompiledRenderPipeline;
use crate::graphics::RenderFeatureDescriptor;

use super::builtin_postprocess_executors::{
    bloom_extract_executor, bloom_postprocess_executor, blur_postprocess_executor,
    clustered_lighting_executor, color_lut_bake_postprocess_executor,
    depth_of_field_postprocess_executor, depth_of_field_prepare_executor,
    exposure_histogram_executor, exposure_resolve_executor, fxaa_postprocess_executor,
    hzb_build_executor, hzb_occlusion_cull_executor, motion_blur_postprocess_executor,
    motion_vector_neighbor_max_executor, motion_vector_tile_max_coarse_executor,
    motion_vector_tile_max_executor, output_transfer_postprocess_executor,
    particle_velocity_executor, scene_composite_postprocess_executor,
    screen_space_reflection_reflection_pyramid_coarse_executor,
    screen_space_reflection_reflection_pyramid_executor, screen_space_reflection_resolve_executor,
    screen_space_reflection_specular_occlusion_executor, smaa_postprocess_executor,
    taa_reactive_mask_mesh_executor, taa_resolve_postprocess_executor, uber_postprocess_executor,
    upscale_postprocess_executor, velocity_camera_executor, velocity_mesh_object_executor,
};
use super::builtin_scene_executors::{
    advanced_pbr_opaque_executor, deferred_gbuffer_executor, deferred_lighting_executor,
    depth_prepass_executor, half_resolution_transparency_composite_executor,
    half_resolution_transparency_depth_downsample_executor, mesh_executor, overlay_gizmo_executor,
    particle_billboard_executor, screen_space_ui_executor, shadow_atlas_executor, sprite_executor,
    transmission_mesh_executor, transmission_scene_copy_executor,
};
use super::generic_compute_executor::generic_compute_executor;
use super::preview_sky_executor::preview_sky_scene_color_executor;
use super::render_pass_executor_registration::{
    render_pass_executor_from_fn, render_pass_executor_from_parallel_safe_fn, RenderPassExecutor,
    RenderPassRecordingPolicy,
};
use super::{RenderPassExecutionContext, RenderPassExecutorId, RenderPassExecutorRegistration};

pub type RenderPassExecutorFn = fn(&mut RenderPassExecutionContext<'_>) -> Result<(), String>;

pub struct RenderPassExecutorRegistry {
    executors: BTreeMap<RenderPassExecutorId, Arc<dyn RenderPassExecutor>>,
    generation: u64,
    last_validated_pipeline_generation: AtomicU64,
    last_validated_registry_generation: AtomicU64,
    #[cfg(test)]
    full_validation_scan_count: AtomicU64,
    #[cfg(test)]
    full_validation_scanned_pass_count: AtomicU64,
}

impl Clone for RenderPassExecutorRegistry {
    fn clone(&self) -> Self {
        Self {
            executors: self.executors.clone(),
            generation: self.generation,
            last_validated_pipeline_generation: AtomicU64::new(0),
            last_validated_registry_generation: AtomicU64::new(0),
            #[cfg(test)]
            full_validation_scan_count: AtomicU64::new(0),
            #[cfg(test)]
            full_validation_scanned_pass_count: AtomicU64::new(0),
        }
    }
}

impl Default for RenderPassExecutorRegistry {
    fn default() -> Self {
        Self {
            executors: BTreeMap::new(),
            generation: 1,
            last_validated_pipeline_generation: AtomicU64::new(0),
            last_validated_registry_generation: AtomicU64::new(0),
            #[cfg(test)]
            full_validation_scan_count: AtomicU64::new(0),
            #[cfg(test)]
            full_validation_scanned_pass_count: AtomicU64::new(0),
        }
    }
}

impl RenderPassExecutorRegistry {
    pub fn with_builtin_noop_executors() -> Self {
        let mut registry = Self::default();
        for executor_id in BUILTIN_NOOP_EXECUTOR_IDS {
            registry.register(
                RenderPassExecutorId::from(*executor_id),
                noop_render_pass_executor,
            );
        }
        registry.register_executor(
            COMPUTE_GENERIC_EXECUTOR_ID.into(),
            generic_compute_executor(),
        );
        registry.register("post.bloom".into(), bloom_postprocess_executor);
        registry.register(
            "post.exposure.histogram".into(),
            exposure_histogram_executor,
        );
        registry.register("post.exposure.resolve".into(), exposure_resolve_executor);
        registry.register(
            "post.color-lut-bake".into(),
            color_lut_bake_postprocess_executor,
        );
        registry.register(
            "post.output-transfer".into(),
            output_transfer_postprocess_executor,
        );
        registry.register("post.upscale".into(), upscale_postprocess_executor);
        registry.register(FXAA_EXECUTOR_ID.into(), fxaa_postprocess_executor);
        registry.register(SMAA_EXECUTOR_ID.into(), smaa_postprocess_executor);
        registry.register_parallel_safe("sprite.opaque".into(), sprite_executor);
        registry.register_parallel_safe("sprite.alpha-mask".into(), sprite_executor);
        registry.register_parallel_safe("sprite.transparent".into(), sprite_executor);
        registry.register_parallel_safe("particle.transparent".into(), particle_billboard_executor);
        registry.register_parallel_safe(
            "particle.halfres-transparent".into(),
            particle_billboard_executor,
        );
        registry.register("mesh.depth-prepass".into(), depth_prepass_executor);
        registry.register("mesh.opaque".into(), mesh_executor);
        registry.register("mesh.alpha-mask".into(), mesh_executor);
        registry.register("mesh.transparent".into(), mesh_executor);
        registry.register("mesh.halfres-transparent".into(), mesh_executor);
        registry.register(
            "transparency.halfres-depth-downsample".into(),
            half_resolution_transparency_depth_downsample_executor,
        );
        registry.register(
            "transparency.halfres-composite".into(),
            half_resolution_transparency_composite_executor,
        );
        registry.register(
            "mesh.advanced-pbr-opaque".into(),
            advanced_pbr_opaque_executor,
        );
        for executor_id in crate::graphics::pipeline::TRANSMISSION_SCENE_COPY_EXECUTOR_IDS {
            registry.register(executor_id.into(), transmission_scene_copy_executor);
        }
        for executor_id in crate::graphics::pipeline::TRANSMISSION_MESH_EXECUTOR_IDS {
            registry.register(executor_id.into(), transmission_mesh_executor);
        }
        registry.register("deferred.depth-prepass".into(), depth_prepass_executor);
        registry.register("deferred.gbuffer".into(), deferred_gbuffer_executor);
        registry.register("lighting.deferred".into(), deferred_lighting_executor);
        registry.register("shadow.atlas".into(), shadow_atlas_executor);
        registry.register(
            "sky.preview-scene-color".into(),
            preview_sky_scene_color_executor,
        );
        registry.register("lighting.light-grid".into(), clustered_lighting_executor);
        registry.register("visibility.hzb-build".into(), hzb_build_executor);
        registry.register(
            "visibility.hzb-occlusion-cull".into(),
            hzb_occlusion_cull_executor,
        );
        registry.register("post.bloom-extract".into(), bloom_extract_executor);
        registry.register("temporal.velocity-camera".into(), velocity_camera_executor);
        registry.register(
            "temporal.velocity-object".into(),
            velocity_mesh_object_executor,
        );
        registry.register("particle.velocity".into(), particle_velocity_executor);
        registry.register(
            "temporal.taa-reactive-mask-mesh".into(),
            taa_reactive_mask_mesh_executor,
        );
        registry.register(
            "temporal.taa-resolve".into(),
            taa_resolve_postprocess_executor,
        );
        registry.register(
            "post.motion-vector-tile-max".into(),
            motion_vector_tile_max_executor,
        );
        registry.register(
            "post.motion-vector-tile-max-coarse".into(),
            motion_vector_tile_max_coarse_executor,
        );
        registry.register(
            "post.motion-vector-neighbor-max".into(),
            motion_vector_neighbor_max_executor,
        );
        registry.register("post.motion-blur".into(), motion_blur_postprocess_executor);
        registry.register("post.blur".into(), blur_postprocess_executor);
        registry.register(
            "post.depth-of-field".into(),
            depth_of_field_postprocess_executor,
        );
        registry.register(
            "post.scene-composite".into(),
            scene_composite_postprocess_executor,
        );
        registry.register(
            "post.depth-of-field-prepare".into(),
            depth_of_field_prepare_executor,
        );
        registry.register(
            "post.screen-space-reflection-reflection-pyramid".into(),
            screen_space_reflection_reflection_pyramid_executor,
        );
        registry.register(
            "post.screen-space-reflection-reflection-pyramid-coarse".into(),
            screen_space_reflection_reflection_pyramid_coarse_executor,
        );
        registry.register(
            "post.screen-space-reflection-resolve".into(),
            screen_space_reflection_resolve_executor,
        );
        registry.register(
            "post.screen-space-reflection-specular-occlusion".into(),
            screen_space_reflection_specular_occlusion_executor,
        );
        registry.register("post.uber".into(), uber_postprocess_executor);
        registry.register("ui.screen-space".into(), screen_space_ui_executor);
        registry.register("overlay.gizmo".into(), overlay_gizmo_executor);
        registry
    }

    pub(crate) fn with_builtin_noop_executors_for_render_features(
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
    ) -> Self {
        let mut registry = Self::with_builtin_noop_executors();
        registry.register_builtin_noop_allowlist_for_render_features(render_features);
        registry
    }

    pub(crate) fn with_builtin_noop_executors_for_render_features_and_executor_registrations(
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        executor_registrations: impl IntoIterator<Item = RenderPassExecutorRegistration>,
    ) -> Self {
        let mut registry = Self::with_builtin_noop_executors_for_render_features(render_features);
        registry.register_explicit_executors(executor_registrations);
        registry
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_environment_ibl_bake_compute_executors(
        mut self,
    ) -> Self {
        self.register_explicit_executors(ibl_bake_compute_executor_registrations());
        self
    }

    fn register_builtin_noop_allowlist_for_render_features(
        &mut self,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
    ) {
        for render_feature in render_features {
            for pass in render_feature.stage_passes {
                registry_register_builtin_noop_executor(self, pass.executor_id);
            }
        }
    }

    pub fn register(
        &mut self,
        id: RenderPassExecutorId,
        executor: RenderPassExecutorFn,
    ) -> Option<Arc<dyn RenderPassExecutor>> {
        self.register_executor(id, render_pass_executor_from_fn(executor))
    }

    fn register_parallel_safe(
        &mut self,
        id: RenderPassExecutorId,
        executor: RenderPassExecutorFn,
    ) -> Option<Arc<dyn RenderPassExecutor>> {
        self.register_executor(id, render_pass_executor_from_parallel_safe_fn(executor))
    }

    pub fn register_executor(
        &mut self,
        id: RenderPassExecutorId,
        executor: Arc<dyn RenderPassExecutor>,
    ) -> Option<Arc<dyn RenderPassExecutor>> {
        let previous = self.executors.insert(id, executor);
        self.advance_generation();
        previous
    }

    pub fn unregister_executor(
        &mut self,
        id: &RenderPassExecutorId,
    ) -> Option<Arc<dyn RenderPassExecutor>> {
        let removed = self.executors.remove(id);
        if removed.is_some() {
            self.advance_generation();
        }
        removed
    }

    pub fn register_explicit_executors(
        &mut self,
        registrations: impl IntoIterator<Item = RenderPassExecutorRegistration>,
    ) {
        for registration in registrations {
            self.register_executor(registration.executor_id, registration.executor);
        }
    }

    #[cfg(test)]
    pub fn contains(&self, id: &RenderPassExecutorId) -> bool {
        self.executors.contains_key(id)
    }

    pub fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        let executor = self.executors.get(&context.executor_id).ok_or_else(|| {
            format!(
                "render pass executor `{}` is not registered",
                context.executor_id
            )
        })?;
        executor.execute(context)
    }

    pub(in crate::graphics::scene::scene_renderer) fn supports_parallel_recording(
        &self,
        executor_id: &str,
    ) -> bool {
        self.executors.get(executor_id).is_some_and(|executor| {
            executor.recording_policy() == RenderPassRecordingPolicy::ParallelSafe
        })
    }

    pub fn validate_compiled_pipeline(
        &self,
        pipeline: &CompiledRenderPipeline,
    ) -> Result<(), String> {
        let pipeline_generation = pipeline.executor_validation_generation();
        if self
            .last_validated_registry_generation
            .load(Ordering::Acquire)
            == self.generation
            && self
                .last_validated_pipeline_generation
                .load(Ordering::Relaxed)
                == pipeline_generation
        {
            return Ok(());
        }

        #[cfg(test)]
        self.full_validation_scan_count
            .fetch_add(1, Ordering::Relaxed);
        for pass in pipeline.graph().passes().iter().filter(|pass| !pass.culled) {
            #[cfg(test)]
            self.full_validation_scanned_pass_count
                .fetch_add(1, Ordering::Relaxed);
            let Some(executor_id) = pass.executor_id.as_ref() else {
                return Err(format!("render pass `{}` has no executor id", pass.name));
            };
            if !self.executors.contains_key(executor_id.as_str()) {
                return Err(format!(
                    "render pass `{}` references unregistered executor `{executor_id}`",
                    pass.name
                ));
            }
        }
        self.last_validated_pipeline_generation
            .store(pipeline_generation, Ordering::Relaxed);
        self.last_validated_registry_generation
            .store(self.generation, Ordering::Release);
        Ok(())
    }

    fn advance_generation(&mut self) {
        self.generation = self.generation.wrapping_add(1).max(1);
        self.last_validated_pipeline_generation
            .store(0, Ordering::Relaxed);
        self.last_validated_registry_generation
            .store(0, Ordering::Release);
    }

    #[cfg(test)]
    fn generation(&self) -> u64 {
        self.generation
    }

    #[cfg(test)]
    fn full_validation_scan_count(&self) -> u64 {
        self.full_validation_scan_count.load(Ordering::Relaxed)
    }

    #[cfg(test)]
    fn full_validation_scanned_pass_count(&self) -> u64 {
        self.full_validation_scanned_pass_count
            .load(Ordering::Relaxed)
    }
}

const BUILTIN_NOOP_EXECUTOR_IDS: &[&str] = &[
    "lighting.baked-composite",
    "lighting.reflection-probes",
    "mesh.alpha-mask",
    "mesh.opaque",
    "mesh.transparent",
];

fn noop_render_pass_executor(_context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
    Ok(())
}

fn registry_register_builtin_noop_executor(
    registry: &mut RenderPassExecutorRegistry,
    executor_id: RenderPassExecutorId,
) {
    // Plugin descriptors declare pass topology only. Unknown executor ids must
    // arrive through explicit plugin registrations instead of being backfilled
    // with a runtime-owned no-op.
    if !BUILTIN_NOOP_EXECUTOR_IDS.contains(&executor_id.as_str()) {
        return;
    }
    if !registry.executors.contains_key(&executor_id) {
        registry.register(executor_id, noop_render_pass_executor);
    }
}

#[cfg(test)]
mod tests;
