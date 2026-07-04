use crate::core::framework::render::{
    AntiAliasSettings, FallbackSkyboxKind, PostProcessGraphResourceNames,
    PostProcessStackDescriptor, PreviewEnvironmentExtract, RenderFrameExtract,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};
use crate::core::math::Vec4;
use crate::graphics::tests::plugin_render_feature_fixtures::{
    hybrid_gi_render_feature_descriptor, virtual_geometry_render_feature_descriptor,
};
use crate::graphics::{
    BuiltinRenderFeature, FrameHistoryBinding, FrameHistorySlot,
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions, RendererFeatureAsset,
};
use crate::render_graph::{
    QueueLane, RenderGraphComputeDispatchExtent, RenderGraphComputeWorkload,
    RenderGraphResourceAccessKind,
};

mod particle;

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
        for feature in [
            BuiltinRenderFeature::PostProcess,
            BuiltinRenderFeature::AntiAlias,
            BuiltinRenderFeature::Ui,
        ] {
            assert!(
                pipeline
                    .renderer
                    .features
                    .iter()
                    .any(|asset| asset.is_builtin(feature)),
                "{} should keep {:?} in the product render graph",
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
        assert!(
            !pipeline
                .renderer
                .features
                .iter()
                .any(|feature| feature.is_builtin(BuiltinRenderFeature::NeuralCompute)),
            "{} should receive neural compute work from plugin descriptors",
            pipeline.name
        );
        for feature in [
            BuiltinRenderFeature::SkinnedMesh,
            BuiltinRenderFeature::Particle,
            BuiltinRenderFeature::Terrain,
            BuiltinRenderFeature::Tree,
            BuiltinRenderFeature::Decal,
            BuiltinRenderFeature::Projector,
            BuiltinRenderFeature::Halo,
            BuiltinRenderFeature::LensFlare,
            BuiltinRenderFeature::Trail,
            BuiltinRenderFeature::Billboard,
            BuiltinRenderFeature::Tilemap,
            BuiltinRenderFeature::TextShaping,
            BuiltinRenderFeature::Skybox,
            BuiltinRenderFeature::Cubemap,
            BuiltinRenderFeature::Texture2dArray,
            BuiltinRenderFeature::NormalMap,
            BuiltinRenderFeature::Mipmap,
            BuiltinRenderFeature::ColorSpace,
        ] {
            assert!(
                !pipeline
                    .renderer
                    .features
                    .iter()
                    .any(|asset| asset.is_builtin(feature)),
                "{} should receive {:?} from later render feature plans",
                pipeline.name,
                feature
            );
        }
    }
}

#[test]
fn compiled_pipeline_collects_enabled_plugin_feature_capability_requirements() {
    let pipeline = RenderPipelineAsset::default_forward_plus().with_plugin_render_features([
        virtual_geometry_render_feature_descriptor(),
        hybrid_gi_render_feature_descriptor(),
    ]);
    let options = RenderPipelineCompileOptions::default()
        .with_feature_disabled(BuiltinRenderFeature::AntiAlias)
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
    let hybrid_trace = enabled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "hybrid-gi-trace-schedule")
        .expect("hybrid GI trace schedule pass should compile");
    let hybrid_workload = hybrid_trace
        .compute_workload
        .as_ref()
        .expect("hybrid GI trace schedule pass should keep planned compute workload metadata");
    assert_eq!(
        hybrid_workload.pipeline_label,
        "zircon-hybrid-gi-trace-schedule"
    );
    assert_eq!(hybrid_workload.workgroup_size, [8, 8, 1]);
    assert_eq!(
        hybrid_workload.dispatch_extent,
        RenderGraphComputeDispatchExtent::Fixed([1, 1, 1])
    );
    let virtual_geometry_cull = enabled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "virtual-geometry-node-cluster-cull")
        .expect("virtual geometry node-cluster cull pass should compile");
    let virtual_geometry_workload = virtual_geometry_cull
        .compute_workload
        .as_ref()
        .expect("virtual geometry cull pass should keep planned compute workload metadata");
    assert_eq!(
        virtual_geometry_workload.pipeline_label,
        "zircon-virtual-geometry-node-cluster-cull"
    );
    assert_eq!(virtual_geometry_workload.workgroup_size, [64, 1, 1]);
    assert_eq!(
        virtual_geometry_workload.dispatch_extent,
        RenderGraphComputeDispatchExtent::Fixed([1, 1, 1])
    );
    assert!(enabled
        .history_bindings
        .contains(&FrameHistoryBinding::read_write(
            FrameHistorySlot::GlobalIllumination
        )));
}

#[test]
fn builtin_smaa_terminal_aa_pass_compiles_after_output_transfer_when_requested() {
    let extract = test_extract();
    let stack = PostProcessStackDescriptor::from_extract_settings_with_anti_alias(
        &extract.post_process.bloom,
        &extract.post_process.color_grading,
        false,
        false,
        &AntiAliasSettings::smaa(),
    );
    let compiled = RenderPipelineAsset::default_forward_plus()
        .compile_with_options(
            &extract,
            &RenderPipelineCompileOptions::default().with_post_process_stack(stack),
        )
        .unwrap();
    let passes = compiled.graph.passes();
    let pass_names = passes
        .iter()
        .map(|pass| pass.name.as_str())
        .collect::<Vec<_>>();

    assert!(pass_names.contains(&"smaa"));
    assert!(!pass_names.contains(&"fxaa"));
    assert!(compiled
        .capability_requirements
        .contains(&RenderFeatureCapabilityRequirement::ScreenSpaceAntiAlias));

    let output_index = pass_names
        .iter()
        .position(|name| *name == "output-transfer")
        .expect("postprocess stack should keep output-transfer before terminal AA");
    let smaa_index = pass_names
        .iter()
        .position(|name| *name == "smaa")
        .expect("SMAA terminal AA pass should compile");
    assert!(output_index < smaa_index);

    let output_transfer = passes
        .iter()
        .find(|pass| pass.name == "output-transfer")
        .expect("postprocess stack should compile output transfer");
    assert!(output_transfer.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::FINAL_COMPOSITED
            && resource.access == RenderGraphResourceAccessKind::Write
    }));
    assert!(!output_transfer.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::FINAL_COLOR
            && resource.access == RenderGraphResourceAccessKind::Write
    }));

    let smaa = passes
        .iter()
        .find(|pass| pass.name == "smaa")
        .expect("SMAA terminal AA pass should compile");
    assert!(smaa.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::FINAL_COMPOSITED
            && resource.access == RenderGraphResourceAccessKind::Read
    }));
    assert!(smaa.resources.iter().any(|resource| {
        resource.name == PostProcessGraphResourceNames::FINAL_COLOR
            && resource.access == RenderGraphResourceAccessKind::Write
    }));
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
fn plugin_neural_compute_feature_respects_capability_opt_in_gate() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::plugin(
            plugin_neural_compute_descriptor(),
        ));

    let disabled = pipeline.compile(&test_extract()).unwrap();
    assert!(
        !disabled
            .graph
            .passes()
            .iter()
            .any(|pass| pass.name == "plugin-neural-inference"),
        "neural compute plugin passes should not compile until the capability is enabled"
    );
    assert!(
        !disabled
            .capability_requirements
            .contains(&RenderFeatureCapabilityRequirement::NeuralCompute),
        "disabled neural plugin descriptors should not add runtime capability requirements"
    );

    let enabled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_capability_enabled(RenderFeatureCapabilityRequirement::NeuralCompute),
        )
        .unwrap();

    let neural_pass = enabled
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == "plugin-neural-inference")
        .expect("enabled neural compute plugin pass should compile into the render graph");
    assert_eq!(neural_pass.queue, QueueLane::AsyncCompute);
    let workload = neural_pass
        .compute_workload
        .as_ref()
        .expect("neural compute pass should keep planned compute workload metadata");
    assert_eq!(workload.pipeline_label, "zircon-neural-inference");
    assert_eq!(workload.workgroup_size, [8, 8, 1]);
    assert_eq!(
        workload.dispatch_extent,
        RenderGraphComputeDispatchExtent::Viewport
    );
    assert!(enabled
        .required_extract_sections
        .contains(&"plugin_neural_compute".to_string()));
    assert!(enabled
        .capability_requirements
        .contains(&RenderFeatureCapabilityRequirement::NeuralCompute));
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

fn plugin_neural_compute_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        "plugin.neural_compute",
        vec!["plugin_neural_compute".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "plugin-neural-inference",
            QueueLane::AsyncCompute,
        )
        .with_executor_id("plugin.neural.inference")
        .with_compute_workload(RenderGraphComputeWorkload::viewport(
            "zircon-neural-inference",
            [8, 8, 1],
        ))
        .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
        .write_buffer("plugin-neural-output")],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::NeuralCompute)
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

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            environment: crate::core::framework::render::EnvironmentExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    )
}
