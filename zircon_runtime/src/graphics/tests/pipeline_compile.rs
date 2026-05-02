use crate::core::framework::render::{RenderFrameExtract, RenderWorldSnapshotHandle};
use crate::graphics::tests::plugin_render_feature_fixtures::{
    hybrid_gi_render_feature_descriptor, virtual_geometry_render_feature_descriptor,
};
use crate::render_graph::{QueueLane, RenderGraphResourceKind};
use crate::scene::world::World;

use crate::{
    BuiltinRenderFeature, FrameHistoryAccess, FrameHistoryBinding, FrameHistorySlot,
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions, RendererFeatureAsset,
};

#[test]
fn default_forward_plus_pipeline_compiles_expected_stage_order_and_passes() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let compiled = pipeline.compile(&test_extract()).unwrap();

    assert_eq!(
        compiled.stages,
        vec![
            RenderPassStage::DepthPrepass,
            RenderPassStage::Shadow,
            RenderPassStage::AmbientOcclusion,
            RenderPassStage::Lighting,
            RenderPassStage::Opaque,
            RenderPassStage::Transparent,
            RenderPassStage::PostProcess,
            RenderPassStage::Overlay,
        ]
    );
    assert_eq!(
        compiled
            .graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "depth-prepass",
            "shadow-map",
            "clustered-light-culling",
            "opaque-mesh",
            "transparent-mesh",
            "bloom-extract",
            "color-grade",
            "history-resolve",
            "overlay-gizmo",
        ]
    );
    assert_eq!(
        compiled.required_extract_sections,
        vec![
            "debug".to_string(),
            "geometry".to_string(),
            "lighting".to_string(),
            "post_process".to_string(),
            "view".to_string(),
            "visibility".to_string(),
        ]
    );
    assert_eq!(
        compiled.history_bindings,
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::SceneColor
        )]
    );
}

#[test]
fn default_deferred_pipeline_compiles_expected_stage_order_and_passes() {
    let pipeline = RenderPipelineAsset::default_deferred();
    let compiled = pipeline.compile(&test_extract()).unwrap();

    assert_eq!(
        compiled.stages,
        vec![
            RenderPassStage::DepthPrepass,
            RenderPassStage::Shadow,
            RenderPassStage::GBuffer,
            RenderPassStage::AmbientOcclusion,
            RenderPassStage::Lighting,
            RenderPassStage::Transparent,
            RenderPassStage::PostProcess,
            RenderPassStage::Overlay,
        ]
    );
    assert_eq!(
        compiled
            .graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "depth-prepass",
            "shadow-map",
            "gbuffer-mesh",
            "clustered-light-culling",
            "deferred-lighting",
            "transparent-mesh",
            "bloom-extract",
            "color-grade",
            "history-resolve",
            "overlay-gizmo",
        ]
    );
    assert_eq!(
        compiled.required_extract_sections,
        vec![
            "debug".to_string(),
            "geometry".to_string(),
            "lighting".to_string(),
            "post_process".to_string(),
            "view".to_string(),
            "visibility".to_string(),
        ]
    );
    assert_eq!(
        compiled.history_bindings,
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::SceneColor
        )]
    );
}

#[test]
fn default_pipeline_assets_do_not_embed_pluginized_advanced_builtin_features() {
    for pipeline in [
        RenderPipelineAsset::default_forward_plus(),
        RenderPipelineAsset::default_deferred(),
    ] {
        for feature in [
            BuiltinRenderFeature::ScreenSpaceAmbientOcclusion,
            BuiltinRenderFeature::ReflectionProbes,
            BuiltinRenderFeature::BakedLighting,
            BuiltinRenderFeature::PostProcess,
        ] {
            assert!(
                !pipeline
                    .renderer
                    .features
                    .iter()
                    .any(|asset| asset.is_builtin(feature)),
                "{} should receive {:?} from rendering plugin descriptors",
                pipeline.name,
                feature
            );
        }
        assert!(
            !pipeline
                .renderer
                .features
                .iter()
                .any(|feature| feature.is_builtin(BuiltinRenderFeature::VirtualGeometry)),
            "{} should receive virtual geometry from plugin descriptors",
            pipeline.name
        );
        assert!(
            !pipeline
                .renderer
                .features
                .iter()
                .any(|feature| feature.is_builtin(BuiltinRenderFeature::GlobalIllumination)),
            "{} should receive hybrid GI from plugin descriptors",
            pipeline.name
        );
        assert!(
            !pipeline
                .renderer
                .features
                .iter()
                .any(|feature| feature.is_builtin(BuiltinRenderFeature::Particle)),
            "{} should receive particles from plugin descriptors",
            pipeline.name
        );
    }
}

#[test]
fn rendering_plugin_default_features_restore_legacy_forward_plus_pass_order() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features(default_rendering_feature_descriptors());
    let compiled = pipeline.compile(&test_extract()).unwrap();

    assert_eq!(
        compiled
            .graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "depth-prepass",
            "shadow-map",
            "ssao-evaluate",
            "clustered-light-culling",
            "opaque-mesh",
            "transparent-mesh",
            "bloom-extract",
            "reflection-probe-composite",
            "baked-lighting-composite",
            "post-process",
            "color-grade",
            "history-resolve",
            "overlay-gizmo",
        ]
    );
    assert_eq!(
        compiled.history_bindings,
        vec![
            FrameHistoryBinding::read_write(FrameHistorySlot::AmbientOcclusion),
            FrameHistoryBinding::read_write(FrameHistorySlot::SceneColor),
        ]
    );
}

#[test]
fn rendering_plugin_default_features_restore_legacy_deferred_pass_order() {
    let pipeline = RenderPipelineAsset::default_deferred()
        .with_plugin_render_features(default_rendering_feature_descriptors());
    let compiled = pipeline.compile(&test_extract()).unwrap();

    assert_eq!(
        compiled
            .graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "depth-prepass",
            "shadow-map",
            "gbuffer-mesh",
            "ssao-evaluate",
            "clustered-light-culling",
            "deferred-lighting",
            "transparent-mesh",
            "bloom-extract",
            "reflection-probe-composite",
            "baked-lighting-composite",
            "post-process",
            "color-grade",
            "history-resolve",
            "overlay-gizmo",
        ]
    );
}

#[test]
fn particle_plugin_render_feature_adds_transparent_pass_to_default_pipeline() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([particle_render_feature_descriptor()]);
    let compiled = pipeline.compile(&test_extract()).unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(pass_names.contains(&"particle-render"));
    assert!(compiled
        .required_extract_sections
        .contains(&"particles".to_string()));
    let particle_feature = compiled
        .enabled_features
        .iter()
        .find(|feature| feature.feature_name() == "particle")
        .expect("particle plugin feature should remain in compiled pipeline");
    assert!(
        particle_feature.builtin_feature().is_none(),
        "particle plugin feature should not reintroduce built-in feature identity"
    );
}

#[test]
fn compile_options_can_disable_particle_plugin_feature_by_name() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([particle_render_feature_descriptor()]);
    let compiled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default().with_plugin_feature_disabled("particle"),
        )
        .unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(!pass_names.contains(&"particle-render"));
    assert!(!compiled
        .required_extract_sections
        .contains(&"particles".to_string()));
}

#[test]
fn forward_plus_pipeline_compilation_is_deterministic() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let extract = test_extract();

    let first = pipeline.compile(&extract).unwrap();
    let second = pipeline.compile(&extract).unwrap();

    assert_eq!(first, second);
}

#[test]
fn builtin_pipeline_lookup_exposes_deferred_pipeline_handle() {
    let builtin =
        RenderPipelineAsset::builtin(crate::core::framework::render::RenderPipelineHandle::new(2))
            .expect("handle 2 should map to the built-in deferred pipeline");

    assert_eq!(builtin, RenderPipelineAsset::default_deferred());
}

#[test]
fn history_binding_accessors_construct_expected_bindings() {
    assert_eq!(
        FrameHistoryBinding::read(FrameHistorySlot::AmbientOcclusion),
        FrameHistoryBinding {
            slot: FrameHistorySlot::AmbientOcclusion,
            access: FrameHistoryAccess::Read,
        }
    );
    assert_eq!(
        FrameHistoryBinding::write(FrameHistorySlot::SceneColor),
        FrameHistoryBinding {
            slot: FrameHistorySlot::SceneColor,
            access: FrameHistoryAccess::Write,
        }
    );
}

#[test]
fn compile_options_can_disable_clustered_history_and_rendering_plugin_features() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features(default_rendering_feature_descriptors());
    let options = RenderPipelineCompileOptions::default()
        .with_feature_disabled(BuiltinRenderFeature::ClusteredLighting)
        .with_feature_disabled(BuiltinRenderFeature::HistoryResolve)
        .with_plugin_feature_disabled("screen_space_ambient_occlusion");

    let compiled = pipeline
        .compile_with_options(&test_extract(), &options)
        .unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(!pass_names.contains(&"ssao-evaluate"));
    assert!(!pass_names.contains(&"clustered-light-culling"));
    assert!(!pass_names.contains(&"history-resolve"));
    assert!(!compiled
        .history_bindings
        .contains(&FrameHistoryBinding::read_write(
            FrameHistorySlot::AmbientOcclusion
        )));
}

#[test]
fn flagship_feature_descriptors_declare_backend_capability_requirements() {
    assert_eq!(
        BuiltinRenderFeature::VirtualGeometry
            .descriptor()
            .capability_requirements,
        vec![RenderFeatureCapabilityRequirement::VirtualGeometry]
    );
    assert_eq!(
        BuiltinRenderFeature::GlobalIllumination
            .descriptor()
            .capability_requirements,
        vec![RenderFeatureCapabilityRequirement::HybridGlobalIllumination]
    );
    assert_eq!(
        BuiltinRenderFeature::RayTracing
            .descriptor()
            .capability_requirements,
        vec![
            RenderFeatureCapabilityRequirement::AccelerationStructures,
            RenderFeatureCapabilityRequirement::RayTracingPipeline,
        ]
    );
}

#[test]
fn compiled_pipeline_collects_enabled_plugin_feature_capability_requirements() {
    let pipeline = RenderPipelineAsset::default_forward_plus().with_plugin_render_features([
        virtual_geometry_render_feature_descriptor(),
        hybrid_gi_render_feature_descriptor(),
    ]);
    let options = RenderPipelineCompileOptions::default()
        .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry)
        .with_capability_enabled(RenderFeatureCapabilityRequirement::HybridGlobalIllumination);

    let compiled = pipeline
        .compile_with_options(&test_extract(), &options)
        .unwrap();

    assert_eq!(
        compiled.capability_requirements,
        vec![
            RenderFeatureCapabilityRequirement::VirtualGeometry,
            RenderFeatureCapabilityRequirement::HybridGlobalIllumination,
        ]
    );
}

#[test]
fn compile_options_fallback_async_compute_passes_to_graphics_queue() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features(default_rendering_feature_descriptors());
    let options = RenderPipelineCompileOptions::default().with_async_compute(false);

    let compiled = pipeline
        .compile_with_options(&test_extract(), &options)
        .unwrap();

    assert_eq!(
        compiled
            .graph
            .passes()
            .iter()
            .filter(|pass| pass.queue == QueueLane::AsyncCompute)
            .count(),
        0
    );
    assert!(compiled
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "ssao-evaluate"
            && pass.queue == QueueLane::Graphics
            && pass.declared_queue == QueueLane::AsyncCompute));
    assert!(compiled
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "clustered-light-culling"
            && pass.queue == QueueLane::Graphics
            && pass.declared_queue == QueueLane::AsyncCompute));
    assert_eq!(compiled.graph.stats().queue_fallback_pass_count, 2);
}

#[test]
fn feature_pass_descriptors_drive_executor_ids_and_resource_graph() {
    let pipeline = RenderPipelineAsset::default_forward_plus();
    let compiled = pipeline.compile(&test_extract()).unwrap();

    let depth_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "depth-prepass")
        .expect("default forward pipeline should include depth prepass");
    assert_eq!(
        depth_pass.executor_id.as_deref(),
        Some("mesh.depth-prepass")
    );

    let opaque_pass = compiled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "opaque-mesh")
        .expect("default forward pipeline should include opaque mesh pass");
    assert_eq!(opaque_pass.executor_id.as_deref(), Some("mesh.opaque"));

    let lifetimes = compiled.graph.resource_lifetimes();
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == "scene-depth" && lifetime.kind == RenderGraphResourceKind::TransientTexture
    }));
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == "scene-color" && lifetime.kind == RenderGraphResourceKind::TransientTexture
    }));
    assert!(lifetimes.iter().any(|lifetime| {
        lifetime.name == "viewport-output" && lifetime.kind == RenderGraphResourceKind::External
    }));
}

#[test]
fn gi_and_virtual_geometry_opt_in_add_feature_runtime_passes_to_graph() {
    let pipeline = RenderPipelineAsset::default_forward_plus().with_plugin_render_features([
        virtual_geometry_render_feature_descriptor(),
        hybrid_gi_render_feature_descriptor(),
    ]);
    let disabled = pipeline.compile(&test_extract()).unwrap();
    let disabled_pass_names = disabled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();
    assert!(!disabled_pass_names.contains(&"hybrid-gi-resolve"));
    assert!(!disabled_pass_names.contains(&"virtual-geometry-node-cluster-cull"));

    let options = RenderPipelineCompileOptions::default()
        .with_capability_enabled(RenderFeatureCapabilityRequirement::HybridGlobalIllumination)
        .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry);
    let enabled = pipeline
        .compile_with_options(&test_extract(), &options)
        .unwrap();
    let enabled_pass_names = enabled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    for pass_name in [
        "hybrid-gi-scene-prepare",
        "hybrid-gi-trace-schedule",
        "hybrid-gi-resolve",
        "hybrid-gi-history",
        "virtual-geometry-prepare",
        "virtual-geometry-node-cluster-cull",
        "virtual-geometry-page-feedback",
        "virtual-geometry-visbuffer",
        "virtual-geometry-debug-overlay",
    ] {
        assert!(
            enabled_pass_names.contains(&pass_name),
            "enabled graph should contain {pass_name}"
        );
    }
    assert!(enabled
        .history_bindings
        .contains(&FrameHistoryBinding::read_write(
            FrameHistorySlot::GlobalIllumination
        )));
}

#[test]
fn plugin_render_feature_asset_compiles_descriptor_without_builtin_feature_identity() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::plugin(
            plugin_virtual_geometry_descriptor(),
        ));

    let compiled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry),
        )
        .unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(
        pass_names.contains(&"plugin-virtual-geometry-prepare"),
        "plugin descriptor pass should be compiled into the render graph"
    );
    let plugin_feature = compiled
        .enabled_features
        .iter()
        .find(|feature| feature.feature_name() == "plugin.virtual_geometry")
        .expect("compiled pipeline should retain the plugin feature name");
    assert!(
        plugin_feature.builtin_feature().is_none(),
        "plugin renderer feature should not masquerade as a built-in feature"
    );
    assert!(compiled
        .required_extract_sections
        .contains(&"plugin_virtual_geometry".to_string()));
    assert!(compiled
        .capability_requirements
        .contains(&RenderFeatureCapabilityRequirement::VirtualGeometry));
}

#[test]
fn plugin_render_feature_asset_respects_capability_opt_in_gate() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::plugin(
            plugin_virtual_geometry_descriptor(),
        ));

    let compiled = pipeline.compile(&test_extract()).unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(
        !pass_names.contains(&"plugin-virtual-geometry-prepare"),
        "advanced plugin descriptor passes should not compile until their capability is enabled"
    );
    assert!(
        !compiled
            .capability_requirements
            .contains(&RenderFeatureCapabilityRequirement::VirtualGeometry),
        "disabled plugin descriptors should not add runtime capability requirements"
    );
}

#[test]
fn plugin_render_feature_descriptors_replace_advanced_builtin_slots() {
    let pipeline = legacy_advanced_builtin_pipeline().with_plugin_render_features([
        replacement_virtual_geometry_descriptor(),
        replacement_hybrid_gi_descriptor(),
    ]);

    assert!(!pipeline
        .renderer
        .features
        .iter()
        .any(|feature| feature.is_builtin(BuiltinRenderFeature::VirtualGeometry)));
    assert!(!pipeline
        .renderer
        .features
        .iter()
        .any(|feature| feature.is_builtin(BuiltinRenderFeature::GlobalIllumination)));
    assert!(pipeline.renderer.features.iter().any(|feature| {
        feature.feature_name() == "virtual_geometry" && feature.builtin_feature().is_none()
    }));
    assert!(pipeline.renderer.features.iter().any(|feature| {
        feature.feature_name() == "hybrid_gi" && feature.builtin_feature().is_none()
    }));

    let compiled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_capability_enabled(RenderFeatureCapabilityRequirement::VirtualGeometry)
                .with_capability_enabled(
                    RenderFeatureCapabilityRequirement::HybridGlobalIllumination,
                ),
        )
        .unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(pass_names.contains(&"plugin-virtual-geometry-replacement"));
    assert!(pass_names.contains(&"plugin-hybrid-gi-replacement"));
    assert!(
        !pass_names.contains(&"virtual-geometry-prepare"),
        "built-in virtual geometry pass should be removed when plugin descriptor replaces the capability"
    );
    assert!(
        !pass_names.contains(&"hybrid-gi-resolve"),
        "built-in hybrid GI pass should be removed when plugin descriptor replaces the capability"
    );
}

#[test]
fn pipeline_compile_rejects_duplicate_plugin_render_feature_names() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::plugin(RenderFeatureDescriptor::new(
            "plugin.duplicate_feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::Overlay,
                "plugin-duplicate-feature-a",
                QueueLane::Graphics,
            )
            .with_executor_id("plugin.duplicate.a")
            .with_side_effects()],
        )));
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::plugin(RenderFeatureDescriptor::new(
            "plugin.duplicate_feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::Overlay,
                "plugin-duplicate-feature-b",
                QueueLane::Graphics,
            )
            .with_executor_id("plugin.duplicate.b")
            .with_side_effects()],
        )));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("duplicate feature `plugin.duplicate_feature`"),
        "unexpected error: {error}"
    );
}

fn plugin_virtual_geometry_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "plugin.virtual_geometry",
        vec!["plugin_virtual_geometry".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::DepthPrepass,
            "plugin-virtual-geometry-prepare",
            QueueLane::Graphics,
        )
        .with_executor_id("plugin.virtual-geometry.prepare")
        .write_buffer("plugin-virtual-geometry-page-requests")],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry)
}

fn replacement_virtual_geometry_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "virtual_geometry",
        Vec::new(),
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::DepthPrepass,
            "plugin-virtual-geometry-replacement",
            QueueLane::Graphics,
        )
        .with_executor_id("plugin.virtual-geometry.replacement")
        .write_buffer("plugin-virtual-geometry-replacement")],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry)
}

fn replacement_hybrid_gi_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "hybrid_gi",
        Vec::new(),
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::GlobalIllumination,
        )],
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Lighting,
            "plugin-hybrid-gi-replacement",
            QueueLane::Graphics,
        )
        .with_executor_id("plugin.hybrid-gi.replacement")
        .write_texture("plugin-hybrid-gi-lighting")],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::HybridGlobalIllumination)
}

fn particle_render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "particle",
        vec![
            "view".to_string(),
            "particles".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Transparent,
            "particle-render",
            QueueLane::Graphics,
        )
        .with_executor_id("particle.transparent")
        .read_texture("scene-depth")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

fn default_rendering_feature_descriptors() -> Vec<RenderFeatureDescriptor> {
    vec![
        rendering_ssao_descriptor(),
        rendering_reflection_probes_descriptor(),
        rendering_baked_lighting_descriptor(),
        rendering_post_process_descriptor(),
    ]
}

fn rendering_ssao_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "screen_space_ambient_occlusion",
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
        ],
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::AmbientOcclusion,
        )],
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::AmbientOcclusion,
            "ssao-evaluate",
            QueueLane::AsyncCompute,
        )
        .with_executor_id("ao.ssao-evaluate")
        .read_texture("scene-depth")
        .write_texture("ambient-occlusion")],
    )
}

fn rendering_reflection_probes_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "reflection_probes",
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "post_process".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "reflection-probe-composite",
            QueueLane::Graphics,
        )
        .with_executor_id("lighting.reflection-probes")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

fn rendering_baked_lighting_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "baked_lighting",
        vec!["lighting".to_string(), "post_process".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "baked-lighting-composite",
            QueueLane::Graphics,
        )
        .with_executor_id("lighting.baked-composite")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

fn rendering_post_process_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "post_process",
        vec!["view".to_string(), "post_process".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "post-process",
            QueueLane::Graphics,
        )
        .with_executor_id("post.stack")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

fn legacy_advanced_builtin_pipeline() -> RenderPipelineAsset {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::builtin(
            BuiltinRenderFeature::VirtualGeometry,
        ));
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::builtin(
            BuiltinRenderFeature::GlobalIllumination,
        ));
    pipeline
}

#[test]
fn pipeline_compile_rejects_duplicate_stage_entries() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline.renderer.stages.push(RenderPassStage::Opaque);

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("duplicate stage"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_duplicate_feature_entries() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("duplicate feature"),
        "unexpected error: {error}"
    );
}

#[test]
fn disabling_post_process_keeps_overlay_extract_requirements_for_debug_gizmos() {
    let pipeline = RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features(default_rendering_feature_descriptors());

    let compiled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default().with_plugin_feature_disabled("post_process"),
        )
        .unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(
        !pass_names.contains(&"post-process"),
        "post-process pass should be removed when the feature is disabled"
    );
    assert!(
        pass_names.contains(&"overlay-gizmo"),
        "overlay stage should remain available for debug and gizmo rendering"
    );
    assert!(
        compiled
            .required_extract_sections
            .contains(&"debug".to_string()),
        "overlay feature should keep requiring debug extract data"
    );
}

#[test]
fn renderer_feature_asset_quality_gate_controls_compiled_passes() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom")
        .quality_gate = Some(BuiltinRenderFeature::RayTracing);

    let without_gate = pipeline.compile(&test_extract()).unwrap();
    assert!(!without_gate
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "bloom-extract"));

    let with_gate = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::RayTracing),
        )
        .unwrap();
    assert!(with_gate
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "bloom-extract"));
}

#[test]
fn pipeline_compile_validates_quality_gated_descriptor_overrides_even_when_gate_is_closed() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_quality_gate(BuiltinRenderFeature::VirtualGeometry)
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-gated-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::GBuffer,
                "bad-gated-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("bad-gated-pass") && error.contains("undeclared stage"),
        "unexpected error: {error}"
    );
}

#[test]
fn renderer_feature_asset_local_config_and_capabilities_survive_compile() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let color_grading = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::ColorGrading))
        .expect("default pipeline should include color grading");
    *color_grading = color_grading
        .clone()
        .with_local_config("variant", "cinematic")
        .with_capability_requirement(RenderFeatureCapabilityRequirement::RayTracingPipeline);

    let compiled = pipeline.compile(&test_extract()).unwrap();
    let compiled_color_grading = compiled
        .enabled_features
        .iter()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::ColorGrading))
        .expect("color grading should remain enabled");

    assert_eq!(
        compiled_color_grading.local_config.get("variant"),
        Some(&"cinematic".to_string())
    );
    assert!(compiled
        .capability_requirements
        .contains(&RenderFeatureCapabilityRequirement::RayTracingPipeline));
}

#[test]
fn renderer_feature_asset_descriptor_override_changes_compiled_graph() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "custom-bloom",
            vec!["custom_post".to_string()],
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "custom-bloom-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")
            .read_texture("scene-color")
            .write_external("viewport-output")],
        ));

    let compiled = pipeline.compile(&test_extract()).unwrap();
    let pass_names = compiled
        .graph
        .passes()
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(!pass_names.contains(&"bloom-extract"));
    assert!(pass_names.contains(&"custom-bloom-pass"));
    assert!(compiled
        .required_extract_sections
        .contains(&"custom_post".to_string()));
    assert!(compiled.graph.resource_lifetimes().iter().any(|lifetime| {
        lifetime.name == "viewport-output" && lifetime.kind == RenderGraphResourceKind::External
    }));
}

#[test]
fn pipeline_compile_rejects_descriptor_passes_for_undeclared_stages() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-stage-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::GBuffer,
                "custom-gbuffer-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")
            .with_side_effects()],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("custom-gbuffer-pass") && error.contains("undeclared stage"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_duplicate_descriptor_pass_names() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "duplicate-pass-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "color-grade",
                QueueLane::Graphics,
            )
            .with_executor_id("post.color-grade")
            .with_side_effects()],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("duplicate render graph pass name `color-grade`"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_conflicting_descriptor_resource_kinds() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-resource-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "bad-resource-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")
            .write_buffer("scene-color")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("resource `scene-color`") && error.contains("conflicting resource kind"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_explicit_external_resource_name_conflicts() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "bad-external-resource-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "bad-external-resource-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")
            .write_external("scene-color")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("resource `scene-color`") && error.contains("explicit external resource"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_empty_descriptor_pass_executor_and_resource_names() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "",
                QueueLane::Graphics,
            )
            .with_executor_id("")
            .write_texture("")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("feature descriptor name must not be empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_empty_descriptor_pass_names_after_descriptor_name_is_valid() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "empty-pass-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("pass name must not be empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_empty_descriptor_executor_and_resource_names() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "empty-resource-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "empty-resource-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("")
            .write_texture("")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("executor id must not be empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn pipeline_compile_rejects_empty_descriptor_resource_names_after_executor_is_valid() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let bloom = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Bloom))
        .expect("default pipeline should include bloom");
    *bloom = bloom
        .clone()
        .with_descriptor_override(RenderFeatureDescriptor::new(
            "empty-resource-feature",
            Vec::new(),
            Vec::new(),
            vec![RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "empty-resource-pass",
                QueueLane::Graphics,
            )
            .with_executor_id("post.stack")
            .write_texture("")],
        ));

    let error = pipeline.compile(&test_extract()).unwrap_err();

    assert!(
        error.contains("resource name must not be empty"),
        "unexpected error: {error}"
    );
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}
