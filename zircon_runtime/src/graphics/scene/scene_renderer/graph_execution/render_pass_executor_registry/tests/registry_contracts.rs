use super::super::noop_render_pass_executor;
use super::*;
use crate::graphics::scene::scene_renderer::environment::ibl_bake_graph_plan::{
    IBL_BAKE_IRRADIANCE_CUBE_EXECUTOR_ID, IBL_BAKE_IRRADIANCE_SH9_EXECUTOR_ID,
    IBL_BAKE_PMREM_EXECUTOR_ID,
};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderPassExecutor, RenderPassRecordingPolicy,
};
#[test]
fn registry_rejects_unregistered_executor_ids() {
    let registry = RenderPassExecutorRegistry::default();
    let error = registry
        .execute(&mut RenderPassExecutionContext::new(
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
        attachment_ops: None,
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
        RenderPassExecutorId::new("compute.generic"),
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
            RenderPassExecutorId::new("lighting.light-grid"),
            QueueLane::Graphics,
            QueueLane::Graphics,
            PassFlags::default(),
            vec![
                RenderPassId::from_index(1, 0),
                RenderPassId::from_index(3, 0),
            ],
            Vec::new(),
        );
    assert_eq!(
        context.dependencies,
        vec![
            RenderPassId::from_index(1, 0),
            RenderPassId::from_index(3, 0)
        ]
    );
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
    let half_resolution_transparency = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default().with_half_resolution_transparency(true),
        )
        .unwrap();

    for pipeline in [&forward, &deferred, &half_resolution_transparency] {
        registry
            .validate_compiled_pipeline(pipeline)
            .expect("builtin registry should cover all compiled executor ids");
        for pass in pipeline.graph().passes() {
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
    ] {
        assert!(
            !registry.contains(&RenderPassExecutorId::new(executor_id)),
            "core built-in registry should not carry pluginized executor `{executor_id}`"
        );
    }
}

#[test]
fn builtin_registry_excludes_environment_ibl_bake_executor_ids_by_default() {
    let registry = RenderPassExecutorRegistry::with_builtin_noop_executors();

    for executor_id in [
        IBL_BAKE_PMREM_EXECUTOR_ID,
        IBL_BAKE_IRRADIANCE_SH9_EXECUTOR_ID,
        IBL_BAKE_IRRADIANCE_CUBE_EXECUTOR_ID,
    ] {
        assert!(
            !registry.contains(&RenderPassExecutorId::new(executor_id)),
            "IBL bake executor `{executor_id}` should require explicit opt-in until real GPU bake scheduling is wired"
        );
    }
}

#[test]
fn explicit_ibl_bake_registry_extension_covers_environment_executor_ids() {
    let registry = RenderPassExecutorRegistry::with_builtin_noop_executors()
        .with_environment_ibl_bake_compute_executors();

    for executor_id in [
        IBL_BAKE_PMREM_EXECUTOR_ID,
        IBL_BAKE_IRRADIANCE_SH9_EXECUTOR_ID,
        IBL_BAKE_IRRADIANCE_CUBE_EXECUTOR_ID,
    ] {
        assert!(
            registry.contains(&RenderPassExecutorId::new(executor_id)),
            "IBL bake executor `{executor_id}` should be available through explicit registry extension"
        );
    }
}

#[test]
fn builtin_registry_covers_product_postprocess_executor_ids() {
    let registry = RenderPassExecutorRegistry::with_builtin_noop_executors();

    for executor_id in [
        "post.bloom",
        "post.bloom-extract",
        "temporal.velocity-camera",
        "temporal.velocity-object",
        "temporal.taa-reactive-mask-mesh",
        "temporal.taa-resolve",
        "post.motion-vector-tile-max",
        "post.motion-vector-tile-max-coarse",
        "post.motion-vector-neighbor-max",
        "post.motion-blur",
        "post.blur",
        "post.depth-of-field",
        "post.depth-of-field-prepare",
        "post.scene-composite",
        "post.exposure.histogram",
        "post.exposure.resolve",
        "post.screen-space-reflection-reflection-pyramid",
        "post.screen-space-reflection-reflection-pyramid-coarse",
        "post.screen-space-reflection-resolve",
        "post.screen-space-reflection-specular-occlusion",
        "visibility.hzb-build",
        "visibility.hzb-occlusion-cull",
        "particle.transparent",
        "particle.halfres-transparent",
        "mesh.halfres-transparent",
        "transparency.halfres-depth-downsample",
        "transparency.halfres-composite",
        "post.color-lut-bake",
        "post.uber",
        "post.upscale",
        "post.output-transfer",
        "post.fxaa",
        "post.smaa",
    ] {
        assert!(
            registry.contains(&RenderPassExecutorId::new(executor_id)),
            "product postprocess executor `{executor_id}` should be registered"
        );
    }
}

#[test]
fn builtin_registry_covers_preview_sky_executor_ids() {
    let registry = RenderPassExecutorRegistry::with_builtin_noop_executors();

    let executor_id = "sky.preview-scene-color";
    assert!(
        registry.contains(&RenderPassExecutorId::new(executor_id)),
        "preview sky executor `{executor_id}` should be registered"
    );
}

#[test]
fn builtin_registry_covers_runtime_ui_executor_id() {
    let registry = RenderPassExecutorRegistry::with_builtin_noop_executors();

    for executor_id in ["ui.screen-space", "overlay.gizmo"] {
        assert!(
            registry.contains(&RenderPassExecutorId::new(executor_id)),
            "`{executor_id}` should be registered as a graph-owned built-in"
        );
    }
}

#[test]
fn registry_invokes_object_backed_executor_with_mutable_context() {
    let mut registry = RenderPassExecutorRegistry::default();
    registry.register_executor(
        RenderPassExecutorId::new("object.executor"),
        Arc::new(ContextMutatingExecutor),
    );
    let mut context = RenderPassExecutionContext::new(
        "object-pass",
        RenderPassExecutorId::new("object.executor"),
    );

    registry.execute(&mut context).unwrap();

    assert_eq!(context.pass_name, "object-pass:executed");
    assert!(!registry.supports_parallel_recording("object.executor"));
}

#[test]
fn parallel_recording_requires_an_explicit_executor_policy() {
    let mut registry = RenderPassExecutorRegistry::default();
    registry.register(
        RenderPassExecutorId::new("custom.function"),
        noop_render_pass_executor,
    );

    assert!(!registry.supports_parallel_recording("custom.function"));
    assert!(!registry.supports_parallel_recording("missing.executor"));

    let registry = RenderPassExecutorRegistry::with_builtin_noop_executors();
    for executor_id in [
        "sprite.opaque",
        "sprite.alpha-mask",
        "sprite.transparent",
        "particle.transparent",
        "particle.halfres-transparent",
    ] {
        assert!(
            registry.supports_parallel_recording(executor_id),
            "audited built-in executor `{executor_id}` should opt into parallel recording"
        );
    }
    for executor_id in [
        "lighting.baked-composite",
        "lighting.light-grid",
        "post.bloom",
    ] {
        assert!(
            !registry.supports_parallel_recording(executor_id),
            "no-op or queue-writing executor `{executor_id}` must remain serial"
        );
    }
}

#[test]
fn object_executor_must_explicitly_opt_into_parallel_recording() {
    struct ExplicitParallelExecutor;

    impl RenderPassExecutor for ExplicitParallelExecutor {
        fn execute(&self, _context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
            Ok(())
        }

        fn recording_policy(&self) -> RenderPassRecordingPolicy {
            RenderPassRecordingPolicy::ParallelSafe
        }
    }

    let mut registry = RenderPassExecutorRegistry::default();
    registry.register_executor(
        RenderPassExecutorId::new("plugin.explicit-parallel"),
        Arc::new(ExplicitParallelExecutor),
    );

    assert!(registry.supports_parallel_recording("plugin.explicit-parallel"));
}

#[test]
fn registry_rejects_compiled_pipeline_with_unknown_executor_id() {
    let mut graph = RenderGraphBuilder::new("custom-pipeline");
    let custom_pass =
        graph.add_pass_with_executor("custom-pass", QueueLane::Graphics, Some("custom.executor"));
    let output = graph.import_external_resource("custom-output");
    graph.write_external(custom_pass, output).unwrap();
    let pipeline = CompiledRenderPipeline::from_parts(
        crate::graphics::pipeline::CompiledRenderPipelineParts {
            handle: RenderPipelineHandle::new(42),
            name: "custom pipeline".to_string(),
            renderer_name: "custom renderer".to_string(),
            stages: Vec::new(),
            pass_stages: Vec::new(),
            enabled_features: Vec::new(),
            required_extract_sections: Vec::new(),
            capability_requirements: Vec::new(),
            history_bindings: Vec::new(),
            environment_ibl_bake_request: None,
            half_resolution_transparency_depth_sigma:
                crate::core::framework::render::DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA,
            graph: graph.compile().unwrap(),
        },
    );

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
    let custom_pass = graph.add_pass("custom-pass", QueueLane::Graphics);
    let output = graph.import_external_resource("custom-output");
    graph.write_external(custom_pass, output).unwrap();
    let pipeline = CompiledRenderPipeline::from_parts(
        crate::graphics::pipeline::CompiledRenderPipelineParts {
            handle: RenderPipelineHandle::new(44),
            name: "custom pipeline".to_string(),
            renderer_name: "custom renderer".to_string(),
            stages: Vec::new(),
            pass_stages: Vec::new(),
            enabled_features: Vec::new(),
            required_extract_sections: Vec::new(),
            capability_requirements: Vec::new(),
            history_bindings: Vec::new(),
            environment_ibl_bake_request: None,
            half_resolution_transparency_depth_sigma:
                crate::core::framework::render::DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA,
            graph: graph.compile().unwrap(),
        },
    );

    let error = RenderPassExecutorRegistry::with_builtin_noop_executors()
        .validate_compiled_pipeline(&pipeline)
        .unwrap_err();

    assert_eq!(error, "render pass `custom-pass` has no executor id");
}

#[test]
fn registry_ignores_culled_pass_with_unknown_executor_id() {
    let mut graph = RenderGraphBuilder::new("custom-pipeline");
    let root = graph.add_pass_with_executor(
        "root-pass",
        QueueLane::Graphics,
        Some("lighting.baked-composite"),
    );
    let output = graph.import_external_resource("custom-output");
    graph.write_external(root, output).unwrap();
    let unused = graph.create_texture(TextureDesc::new(
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
    let pipeline = CompiledRenderPipeline::from_parts(
        crate::graphics::pipeline::CompiledRenderPipelineParts {
            handle: RenderPipelineHandle::new(43),
            name: "custom pipeline".to_string(),
            renderer_name: "custom renderer".to_string(),
            stages: Vec::new(),
            pass_stages: Vec::new(),
            enabled_features: Vec::new(),
            required_extract_sections: Vec::new(),
            capability_requirements: Vec::new(),
            history_bindings: Vec::new(),
            environment_ibl_bake_request: None,
            half_resolution_transparency_depth_sigma:
                crate::core::framework::render::DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA,
            graph: compiled_graph,
        },
    );

    RenderPassExecutorRegistry::with_builtin_noop_executors()
        .validate_compiled_pipeline(&pipeline)
        .expect("culled passes should not require executor registration");
}
