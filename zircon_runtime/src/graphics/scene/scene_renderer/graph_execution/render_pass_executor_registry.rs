use std::collections::BTreeMap;

use crate::graphics::RenderFeatureDescriptor;
use crate::CompiledRenderPipeline;

use super::{RenderPassExecutionContext, RenderPassExecutorId, RenderPassExecutorRegistration};

pub type RenderPassExecutorFn = fn(&RenderPassExecutionContext) -> Result<(), String>;

#[derive(Clone, Default)]
pub struct RenderPassExecutorRegistry {
    executors: BTreeMap<RenderPassExecutorId, RenderPassExecutorFn>,
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
        registry
    }

    pub fn with_builtin_noop_executors_for_render_features(
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
    ) -> Self {
        let mut registry = Self::with_builtin_noop_executors();
        registry.register_noop_executors_for_render_features(render_features);
        registry
    }

    pub fn with_builtin_noop_executors_for_render_features_and_executor_registrations(
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        executor_registrations: impl IntoIterator<Item = RenderPassExecutorRegistration>,
    ) -> Self {
        let mut registry = Self::with_builtin_noop_executors_for_render_features(render_features);
        registry.register_explicit_executors(executor_registrations);
        registry
    }

    pub fn register_noop_executors_for_render_features(
        &mut self,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
    ) {
        for render_feature in render_features {
            for pass in render_feature.stage_passes {
                registry_register_noop_executor(self, pass.executor_id);
            }
        }
    }

    pub fn register(
        &mut self,
        id: RenderPassExecutorId,
        executor: RenderPassExecutorFn,
    ) -> Option<RenderPassExecutorFn> {
        self.executors.insert(id, executor)
    }

    pub fn register_explicit_executors(
        &mut self,
        registrations: impl IntoIterator<Item = RenderPassExecutorRegistration>,
    ) {
        for registration in registrations {
            self.register(registration.executor_id, registration.executor);
        }
    }

    #[cfg(test)]
    pub fn contains(&self, id: &RenderPassExecutorId) -> bool {
        self.executors.contains_key(id)
    }

    pub fn execute(&self, context: &RenderPassExecutionContext) -> Result<(), String> {
        let executor = self.executors.get(&context.executor_id).ok_or_else(|| {
            format!(
                "render pass executor `{}` is not registered",
                context.executor_id
            )
        })?;
        executor(context)
    }

    pub fn validate_compiled_pipeline(
        &self,
        pipeline: &CompiledRenderPipeline,
    ) -> Result<(), String> {
        for pass in pipeline.graph.passes() {
            let Some(executor_id) = pass.executor_id.as_ref() else {
                if !pass.culled {
                    return Err(format!("render pass `{}` has no executor id", pass.name));
                }
                continue;
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
    "ao.ssao-evaluate",
    "deferred.depth-prepass",
    "deferred.gbuffer",
    "deferred.transparent",
    "history.scene-color",
    "lighting.baked-composite",
    "lighting.clustered-cull",
    "lighting.deferred",
    "lighting.reflection-probes",
    "mesh.depth-prepass",
    "mesh.opaque",
    "mesh.transparent",
    "overlay.gizmo",
    "post.bloom-extract",
    "post.color-grade",
    "post.stack",
    "shadow.map",
];

fn noop_render_pass_executor(_context: &RenderPassExecutionContext) -> Result<(), String> {
    Ok(())
}

fn registry_register_noop_executor(
    registry: &mut RenderPassExecutorRegistry,
    executor_id: RenderPassExecutorId,
) {
    registry.register(executor_id, noop_render_pass_executor);
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        RenderFrameExtract, RenderPipelineHandle, RenderWorldSnapshotHandle,
    };
    use crate::render_graph::{
        PassFlags, QueueLane, RenderGraphBuilder, RenderGraphPassResourceAccess,
        RenderGraphResourceAccessKind, RenderGraphResourceKind, RenderPassId,
    };
    use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};
    use crate::scene::world::World;
    use crate::{
        CompiledRenderPipeline, RenderFeatureCapabilityRequirement, RenderFeatureDescriptor,
        RenderFeaturePassDescriptor, RenderPassStage, RenderPipelineAsset,
        RenderPipelineCompileOptions, RendererFeatureAsset,
    };

    use super::super::{
        RenderPassExecutionContext, RenderPassExecutorId, RenderPassExecutorRegistration,
    };
    use super::RenderPassExecutorRegistry;

    #[test]
    fn registry_rejects_unregistered_executor_ids() {
        let registry = RenderPassExecutorRegistry::default();
        let error = registry
            .execute(&RenderPassExecutionContext::new(
                "custom-pass",
                RenderPassExecutorId::new("custom.executor"),
            ))
            .unwrap_err();

        assert_eq!(
            error,
            "render pass executor `custom.executor` is not registered"
        );
    }

    #[test]
    fn execution_context_records_graph_queue_and_pass_flags() {
        let context = RenderPassExecutionContext::with_graph_metadata(
            "async-virtual-geometry-cull",
            RenderPassExecutorId::new("virtual-geometry.node-cluster-cull"),
            QueueLane::AsyncCompute,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        );

        assert_eq!(context.declared_queue, QueueLane::AsyncCompute);
        assert_eq!(context.queue, QueueLane::AsyncCompute);
        assert_eq!(
            context.flags,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            }
        );
        assert!(context.resources.is_empty());

        let resources = vec![RenderGraphPassResourceAccess {
            name: "virtual-geometry-visible-clusters".to_string(),
            kind: RenderGraphResourceKind::TransientBuffer,
            access: RenderGraphResourceAccessKind::Read,
        }];
        let context = RenderPassExecutionContext::with_graph_metadata_and_resources(
            "async-virtual-geometry-visbuffer",
            RenderPassExecutorId::new("virtual-geometry.visbuffer"),
            QueueLane::Graphics,
            PassFlags::default(),
            resources.clone(),
        );

        assert_eq!(context.resources, resources);
        assert!(context.dependencies.is_empty());
        assert!(!context.uses_queue_fallback());

        let context = RenderPassExecutionContext::with_declared_graph_metadata(
            "fallback-ssao",
            RenderPassExecutorId::new("ao.ssao-evaluate"),
            QueueLane::Graphics,
            QueueLane::AsyncCompute,
            PassFlags::default(),
        );
        assert_eq!(context.queue, QueueLane::Graphics);
        assert_eq!(context.declared_queue, QueueLane::AsyncCompute);
        assert!(context.uses_queue_fallback());

        let context =
            RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
                "lighting",
                RenderPassExecutorId::new("lighting.clustered-cull"),
                QueueLane::Graphics,
                QueueLane::Graphics,
                PassFlags::default(),
                vec![RenderPassId(1), RenderPassId(3)],
                Vec::new(),
            );
        assert_eq!(context.dependencies, vec![RenderPassId(1), RenderPassId(3)]);
    }

    #[test]
    fn builtin_registry_covers_compiled_pipeline_executor_ids() {
        let registry = RenderPassExecutorRegistry::with_builtin_noop_executors();
        let extract = test_extract();
        let forward = RenderPipelineAsset::default_forward_plus()
            .compile_with_options(&extract, &RenderPipelineCompileOptions::default())
            .unwrap();
        let deferred = RenderPipelineAsset::default_deferred()
            .compile_with_options(&extract, &RenderPipelineCompileOptions::default())
            .unwrap();

        for pipeline in [&forward, &deferred] {
            registry
                .validate_compiled_pipeline(pipeline)
                .expect("builtin registry should cover all compiled executor ids");
            for pass in pipeline.graph.passes() {
                let executor_id = pass
                    .executor_id
                    .as_ref()
                    .expect("compiled SRP passes should carry executor ids");
                assert!(
                    registry.contains(&RenderPassExecutorId::new(executor_id.clone())),
                    "builtin registry should cover executor `{executor_id}` for pass `{}`",
                    pass.name
                );
            }
        }
    }

    #[test]
    fn builtin_registry_excludes_pluginized_advanced_executor_ids() {
        let registry = RenderPassExecutorRegistry::with_builtin_noop_executors();

        for executor_id in [
            "virtual-geometry.prepare",
            "virtual-geometry.node-cluster-cull",
            "virtual-geometry.page-feedback",
            "virtual-geometry.visbuffer",
            "virtual-geometry.debug-overlay",
            "hybrid-gi.scene-prepare",
            "hybrid-gi.trace-schedule",
            "hybrid-gi.resolve",
            "hybrid-gi.history",
            "particle.transparent",
        ] {
            assert!(
                !registry.contains(&RenderPassExecutorId::new(executor_id)),
                "core built-in registry should not carry pluginized executor `{executor_id}`"
            );
        }
    }

    #[test]
    fn plugin_render_feature_descriptors_register_noop_executor_ids() {
        let mut pipeline = RenderPipelineAsset::default_forward_plus();
        let descriptor = plugin_virtual_geometry_descriptor();
        pipeline
            .renderer
            .features
            .push(RendererFeatureAsset::plugin(descriptor.clone()));
        let compiled = pipeline
            .compile_with_options(
                &test_extract(),
                &RenderPipelineCompileOptions::default()
                    .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry),
            )
            .unwrap();

        let core_registry = RenderPassExecutorRegistry::with_builtin_noop_executors();
        let error = core_registry
            .validate_compiled_pipeline(&compiled)
            .unwrap_err();
        assert!(
            error.contains("virtual-geometry.prepare"),
            "core registry should reject plugin executor ids before plugin registration: {error}"
        );

        let plugin_registry =
            RenderPassExecutorRegistry::with_builtin_noop_executors_for_render_features([
                descriptor,
            ]);
        plugin_registry
            .validate_compiled_pipeline(&compiled)
            .expect("plugin render feature descriptors should register their executor ids");
    }

    #[test]
    fn explicit_executor_registration_replaces_descriptor_noop_executor() {
        let descriptor = plugin_virtual_geometry_descriptor();
        let registry = RenderPassExecutorRegistry::with_builtin_noop_executors_for_render_features_and_executor_registrations(
            [descriptor],
            [RenderPassExecutorRegistration::new(
                "virtual-geometry.prepare",
                explicit_virtual_geometry_executor,
            )],
        );

        let error = registry
            .execute(&RenderPassExecutionContext::new(
                "plugin-virtual-geometry-registry",
                RenderPassExecutorId::new("virtual-geometry.prepare"),
            ))
            .unwrap_err();

        assert_eq!(error, "explicit virtual geometry executor called");
    }

    #[test]
    fn registry_rejects_compiled_pipeline_with_unknown_executor_id() {
        let mut graph = RenderGraphBuilder::new("custom-pipeline");
        graph.add_pass_with_executor("custom-pass", QueueLane::Graphics, Some("custom.executor"));
        let pipeline = CompiledRenderPipeline {
            handle: RenderPipelineHandle::new(42),
            name: "custom pipeline".to_string(),
            renderer_name: "custom renderer".to_string(),
            stages: Vec::new(),
            enabled_features: Vec::new(),
            required_extract_sections: Vec::new(),
            capability_requirements: Vec::new(),
            history_bindings: Vec::new(),
            graph: graph.compile().unwrap(),
        };

        let error = RenderPassExecutorRegistry::with_builtin_noop_executors()
            .validate_compiled_pipeline(&pipeline)
            .unwrap_err();

        assert_eq!(
            error,
            "render pass `custom-pass` references unregistered executor `custom.executor`"
        );
    }

    #[test]
    fn registry_rejects_executable_compiled_pipeline_pass_without_executor_id() {
        let mut graph = RenderGraphBuilder::new("custom-pipeline");
        graph.add_pass("custom-pass", QueueLane::Graphics);
        let pipeline = CompiledRenderPipeline {
            handle: RenderPipelineHandle::new(44),
            name: "custom pipeline".to_string(),
            renderer_name: "custom renderer".to_string(),
            stages: Vec::new(),
            enabled_features: Vec::new(),
            required_extract_sections: Vec::new(),
            capability_requirements: Vec::new(),
            history_bindings: Vec::new(),
            graph: graph.compile().unwrap(),
        };

        let error = RenderPassExecutorRegistry::with_builtin_noop_executors()
            .validate_compiled_pipeline(&pipeline)
            .unwrap_err();

        assert_eq!(error, "render pass `custom-pass` has no executor id");
    }

    #[test]
    fn registry_rejects_culled_pass_with_unknown_executor_id() {
        let mut graph = RenderGraphBuilder::new("custom-pipeline");
        let unused = graph.create_transient_texture(TextureDesc::new(
            "unused-target",
            1,
            1,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT,
        ));
        let pass =
            graph.add_pass_with_executor("culled-pass", QueueLane::Graphics, Some("custom.culled"));
        graph.write_texture(pass, unused).unwrap();
        let compiled_graph = graph.compile().unwrap();
        assert!(
            compiled_graph
                .passes()
                .iter()
                .any(|pass| pass.name == "culled-pass" && pass.culled),
            "test fixture should produce a culled pass"
        );
        let pipeline = CompiledRenderPipeline {
            handle: RenderPipelineHandle::new(43),
            name: "custom pipeline".to_string(),
            renderer_name: "custom renderer".to_string(),
            stages: Vec::new(),
            enabled_features: Vec::new(),
            required_extract_sections: Vec::new(),
            capability_requirements: Vec::new(),
            history_bindings: Vec::new(),
            graph: compiled_graph,
        };

        let error = RenderPassExecutorRegistry::with_builtin_noop_executors()
            .validate_compiled_pipeline(&pipeline)
            .unwrap_err();

        assert_eq!(
            error,
            "render pass `culled-pass` references unregistered executor `custom.culled`"
        );
    }

    fn test_extract() -> RenderFrameExtract {
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        )
    }

    fn plugin_virtual_geometry_descriptor() -> RenderFeatureDescriptor {
        RenderFeatureDescriptor::new(
            "plugin.virtual_geometry.registry",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "plugin-virtual-geometry-registry",
                QueueLane::Graphics,
            )
            .with_executor_id("virtual-geometry.prepare")
            .with_side_effects()],
        )
        .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry)
    }

    fn explicit_virtual_geometry_executor(
        context: &RenderPassExecutionContext,
    ) -> Result<(), String> {
        if context.executor_id.as_str() == "virtual-geometry.prepare" {
            return Err("explicit virtual geometry executor called".to_string());
        }
        Ok(())
    }
}
