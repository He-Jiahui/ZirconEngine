use crate::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::Vec4;
use crate::graphics::feature::descriptor_only_advanced_slots;
use crate::graphics::{
    BuiltinRenderFeature, RenderFeatureCapabilityRequirement, RenderPipelineAsset,
    RenderPipelineCompileOptions, RendererFeatureAsset,
};

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
    assert_eq!(
        BuiltinRenderFeature::NeuralCompute
            .descriptor()
            .capability_requirements,
        vec![RenderFeatureCapabilityRequirement::NeuralCompute]
    );
    assert_eq!(
        BuiltinRenderFeature::SparseTexture
            .descriptor()
            .capability_requirements,
        vec![RenderFeatureCapabilityRequirement::SparseTexture]
    );
}

#[test]
fn advanced_followup_feature_slots_reserve_extract_sections_without_runtime_passes() {
    for slot in descriptor_only_advanced_slots() {
        let feature = slot.feature();
        let descriptor = feature.descriptor();
        assert_eq!(descriptor.name, slot.descriptor_name());
        assert_eq!(
            descriptor.required_extract_sections,
            slot.extract_section()
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            descriptor.capability_requirements,
            slot.capability_requirement()
                .into_iter()
                .collect::<Vec<_>>()
        );
        assert!(descriptor.history_bindings.is_empty());
        assert!(
            descriptor.stage_passes.is_empty(),
            "{feature:?} should stay descriptor-only until its dedicated render plan registers passes"
        );
        assert!(
            feature.requires_explicit_opt_in(),
            "{feature:?} should not enter default pipelines without explicit opt-in"
        );
    }
}

#[test]
fn neural_compute_builtin_slot_compiles_only_with_explicit_feature_opt_in() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::builtin(
            BuiltinRenderFeature::NeuralCompute,
        ));

    let default_compiled = pipeline.compile(&test_extract()).unwrap();
    assert!(
        !default_compiled
            .capability_requirements
            .contains(&RenderFeatureCapabilityRequirement::NeuralCompute),
        "neural compute should not declare backend requirements until the slot is opted in"
    );

    let enabled_compiled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::NeuralCompute),
        )
        .unwrap();

    assert!(
        enabled_compiled
            .capability_requirements
            .contains(&RenderFeatureCapabilityRequirement::NeuralCompute)
    );
    assert!(
        !enabled_compiled
            .graph()
            .passes()
            .iter()
            .any(|pass| pass.name.contains("neural")),
        "the runtime slot should only declare the capability; plugin descriptors own executable neural passes"
    );
}

#[test]
fn advanced_followup_builtin_slots_compile_only_with_explicit_feature_opt_in() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    for slot in descriptor_only_advanced_slots() {
        pipeline
            .renderer
            .features
            .push(RendererFeatureAsset::builtin(slot.feature()));
    }

    let default_compiled = pipeline.compile(&test_extract()).unwrap();
    for slot in descriptor_only_advanced_slots() {
        let feature = slot.feature();
        assert!(
            !default_compiled
                .enabled_features()
                .iter()
                .any(|asset| asset.is_builtin(feature)),
            "{feature:?} should not compile until explicitly opted in"
        );
        if let Some(section) = slot.extract_section() {
            assert!(
                !default_compiled
                    .required_extract_sections
                    .contains(&section.to_string()),
                "{feature:?} should not request extract data until explicitly opted in"
            );
        }
        if let Some(requirement) = slot.capability_requirement() {
            assert!(
                !default_compiled
                    .capability_requirements
                    .contains(&requirement),
                "{feature:?} should not require backend capability until explicitly opted in"
            );
        }
    }

    let mut options = RenderPipelineCompileOptions::default();
    for slot in descriptor_only_advanced_slots() {
        options = options.with_feature_enabled(slot.feature());
        if slot.requires_capability_opt_in() {
            options = options.with_capability_enabled(
                slot.capability_requirement()
                    .expect("capability opt-in slots must declare a capability requirement"),
            );
        }
    }
    let enabled_compiled = pipeline
        .compile_with_options(&test_extract(), &options)
        .unwrap();

    for slot in descriptor_only_advanced_slots() {
        let feature = slot.feature();
        assert!(
            enabled_compiled
                .enabled_features()
                .iter()
                .any(|asset| asset.is_builtin(feature)),
            "{feature:?} should compile when explicitly opted in"
        );
        if let Some(section) = slot.extract_section() {
            assert!(
                enabled_compiled
                    .required_extract_sections
                    .contains(&section.to_string()),
                "{feature:?} should reserve its neutral extract section"
            );
        }
        if let Some(requirement) = slot.capability_requirement() {
            assert!(
                enabled_compiled
                    .capability_requirements
                    .contains(&requirement),
                "{feature:?} should declare its backend capability requirement"
            );
        }
    }
    assert_eq!(
        enabled_compiled
            .graph()
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        default_compiled
            .graph()
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        "descriptor-only slots must not add executable graph passes"
    );
    let added_requirements = enabled_compiled
        .capability_requirements
        .iter()
        .filter(|requirement| {
            !default_compiled
                .capability_requirements
                .contains(requirement)
        })
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(
        added_requirements,
        vec![RenderFeatureCapabilityRequirement::SparseTexture],
        "only sparse texture should add a backend capability requirement in this follow-up slot set"
    );
}

#[test]
fn sparse_texture_builtin_slot_requires_feature_and_capability_opt_in() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    pipeline
        .renderer
        .features
        .push(RendererFeatureAsset::builtin(
            BuiltinRenderFeature::SparseTexture,
        ));

    let feature_only = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::SparseTexture),
        )
        .unwrap();
    assert!(
        !feature_only
            .enabled_features()
            .iter()
            .any(|feature| feature.is_builtin(BuiltinRenderFeature::SparseTexture)),
        "feature opt-in without the sparse texture capability should keep the slot out of the graph"
    );
    assert!(
        !feature_only
            .capability_requirements
            .contains(&RenderFeatureCapabilityRequirement::SparseTexture)
    );

    let capability_enabled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::SparseTexture)
                .with_capability_enabled(RenderFeatureCapabilityRequirement::SparseTexture),
        )
        .unwrap();
    assert!(
        capability_enabled
            .enabled_features()
            .iter()
            .any(|feature| feature.is_builtin(BuiltinRenderFeature::SparseTexture))
    );
    assert!(
        capability_enabled
            .required_extract_sections
            .contains(&"sparse_texture".to_string())
    );
    assert!(
        capability_enabled
            .capability_requirements
            .contains(&RenderFeatureCapabilityRequirement::SparseTexture)
    );
    assert!(
        !capability_enabled
            .graph()
            .passes()
            .iter()
            .any(|pass| pass.name.contains("sparse")),
        "the runtime slot should only reserve extract/capability; executable sparse passes are follow-up work"
    );
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
