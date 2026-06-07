use crate::core::framework::render::{
    FallbackSkyboxKind, PostProcessGraphResourceNames, PreviewEnvironmentExtract,
    RenderFrameExtract, RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};
use crate::core::math::Vec4;
use crate::graphics::tests::plugin_render_feature_fixtures::{
    hybrid_gi_render_feature_descriptor, particle_render_feature_descriptor,
    virtual_geometry_render_feature_descriptor,
};
use crate::render_graph::QueueLane;
use crate::{
    BuiltinRenderFeature, FrameHistoryBinding, FrameHistorySlot,
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions, RendererFeatureAsset,
};

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
