use std::collections::BTreeMap;

use crate::CompiledRenderPipeline;

use super::{RenderPassExecutionContext, RenderPassExecutorId};

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

    pub fn register(
        &mut self,
        id: RenderPassExecutorId,
        executor: RenderPassExecutorFn,
    ) -> Option<RenderPassExecutorFn> {
        self.executors.insert(id, executor)
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
    "hybrid-gi.history",
    "hybrid-gi.resolve",
    "hybrid-gi.scene-prepare",
    "hybrid-gi.trace-schedule",
    "lighting.baked-composite",
    "lighting.clustered-cull",
    "lighting.deferred",
    "lighting.reflection-probes",
    "mesh.depth-prepass",
    "mesh.opaque",
    "mesh.transparent",
    "overlay.gizmo",
    "particle.transparent",
    "post.bloom-extract",
    "post.color-grade",
    "post.stack",
    "shadow.map",
    "virtual-geometry.debug-overlay",
    "virtual-geometry.node-cluster-cull",
    "virtual-geometry.page-feedback",
    "virtual-geometry.prepare",
    "virtual-geometry.visbuffer",
];

fn noop_render_pass_executor(_context: &RenderPassExecutionContext) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        RenderFrameExtract, RenderPipelineHandle, RenderWorldSnapshotHandle,
    };
    use crate::render_graph::{QueueLane, RenderGraphBuilder};
    use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};
    use crate::scene::world::World;
    use crate::{
        BuiltinRenderFeature, CompiledRenderPipeline, RenderPipelineAsset,
        RenderPipelineCompileOptions,
    };

    use super::super::{RenderPassExecutionContext, RenderPassExecutorId};
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
    fn builtin_registry_covers_compiled_pipeline_executor_ids() {
        let registry = RenderPassExecutorRegistry::with_builtin_noop_executors();
        let extract = test_extract();
        let forward = RenderPipelineAsset::default_forward_plus()
            .compile_with_options(
                &extract,
                &RenderPipelineCompileOptions::default()
                    .with_feature_enabled(BuiltinRenderFeature::VirtualGeometry)
                    .with_feature_enabled(BuiltinRenderFeature::GlobalIllumination),
            )
            .unwrap();
        let deferred = RenderPipelineAsset::default_deferred()
            .compile_with_options(
                &extract,
                &RenderPipelineCompileOptions::default()
                    .with_feature_enabled(BuiltinRenderFeature::VirtualGeometry)
                    .with_feature_enabled(BuiltinRenderFeature::GlobalIllumination),
            )
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
}
