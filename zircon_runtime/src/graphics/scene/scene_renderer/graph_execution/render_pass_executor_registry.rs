use std::collections::BTreeMap;
use std::sync::Arc;

use crate::graphics::scene::anti_alias::fxaa::FXAA_EXECUTOR_ID;
use crate::graphics::RenderFeatureDescriptor;
use crate::CompiledRenderPipeline;

use super::builtin_postprocess_executors::{
    bloom_extract_executor, bloom_postprocess_executor, clustered_lighting_executor,
    color_grading_postprocess_executor, effect_stack_postprocess_executor,
    final_composite_postprocess_executor, fxaa_postprocess_executor,
    history_resolve_postprocess_executor, post_stack_executor, ssao_executor,
};
use super::builtin_scene_executors::{
    deferred_gbuffer_executor, deferred_lighting_executor, depth_prepass_executor, mesh_executor,
    overlay_gizmo_executor, screen_space_ui_executor, sprite_executor,
};
use super::preview_sky_executor::{
    preview_sky_final_color_executor, preview_sky_scene_color_executor,
};
use super::render_pass_executor_registration::{render_pass_executor_from_fn, RenderPassExecutor};
use super::{RenderPassExecutionContext, RenderPassExecutorId, RenderPassExecutorRegistration};

pub type RenderPassExecutorFn = fn(&mut RenderPassExecutionContext<'_>) -> Result<(), String>;

#[derive(Clone, Default)]
pub struct RenderPassExecutorRegistry {
    executors: BTreeMap<RenderPassExecutorId, Arc<dyn RenderPassExecutor>>,
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
        registry.register("post.bloom".into(), bloom_postprocess_executor);
        registry.register(
            "post.color-grade".into(),
            color_grading_postprocess_executor,
        );
        registry.register(
            "post.history-resolve".into(),
            history_resolve_postprocess_executor,
        );
        registry.register(
            "post.effect-stack".into(),
            effect_stack_postprocess_executor,
        );
        registry.register(
            "post.final-composite".into(),
            final_composite_postprocess_executor,
        );
        registry.register(FXAA_EXECUTOR_ID.into(), fxaa_postprocess_executor);
        registry.register("sprite.opaque".into(), sprite_executor);
        registry.register("sprite.alpha-mask".into(), sprite_executor);
        registry.register("sprite.transparent".into(), sprite_executor);
        registry.register("mesh.depth-prepass".into(), depth_prepass_executor);
        registry.register("mesh.opaque".into(), mesh_executor);
        registry.register("mesh.alpha-mask".into(), mesh_executor);
        registry.register("mesh.transparent".into(), mesh_executor);
        registry.register("deferred.depth-prepass".into(), depth_prepass_executor);
        registry.register("deferred.gbuffer".into(), deferred_gbuffer_executor);
        registry.register("lighting.deferred".into(), deferred_lighting_executor);
        registry.register(
            "sky.preview-scene-color".into(),
            preview_sky_scene_color_executor,
        );
        registry.register(
            "sky.preview-final-color".into(),
            preview_sky_final_color_executor,
        );
        registry.register("ao.ssao-evaluate".into(), ssao_executor);
        registry.register(
            "lighting.clustered-cull".into(),
            clustered_lighting_executor,
        );
        registry.register("post.bloom-extract".into(), bloom_extract_executor);
        registry.register("post.stack".into(), post_stack_executor);
        registry.register("ui.screen-space".into(), screen_space_ui_executor);
        registry.register("overlay.gizmo".into(), overlay_gizmo_executor);
        registry.register(
            "history.scene-color".into(),
            history_resolve_postprocess_executor,
        );
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

    pub fn register_executor(
        &mut self,
        id: RenderPassExecutorId,
        executor: Arc<dyn RenderPassExecutor>,
    ) -> Option<Arc<dyn RenderPassExecutor>> {
        self.executors.insert(id, executor)
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

    pub fn validate_compiled_pipeline(
        &self,
        pipeline: &CompiledRenderPipeline,
    ) -> Result<(), String> {
        for pass in pipeline.graph.passes().iter().filter(|pass| !pass.culled) {
            let Some(executor_id) = pass.executor_id.as_ref() else {
                return Err(format!("render pass `{}` has no executor id", pass.name));
            };
            let executor_id = RenderPassExecutorId::new(executor_id.clone());
            if !self.executors.contains_key(&executor_id) {
                return Err(format!(
                    "render pass `{}` references unregistered executor `{executor_id}`",
                    pass.name
                ));
            }
        }
        Ok(())
    }
}

const BUILTIN_NOOP_EXECUTOR_IDS: &[&str] = &[
    "history.scene-color",
    "lighting.baked-composite",
    "lighting.reflection-probes",
    "mesh.alpha-mask",
    "mesh.opaque",
    "mesh.transparent",
    "post.color-grade",
    "shadow.map",
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
