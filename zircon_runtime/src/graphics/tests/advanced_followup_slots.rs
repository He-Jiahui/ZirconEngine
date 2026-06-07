use crate::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::Vec4;
use crate::{
    BuiltinRenderFeature, RenderFeatureCapabilityRequirement, RenderPipelineAsset,
    RenderPipelineCompileOptions, RendererFeatureAsset,
};

const ADVANCED_CAPABILITY_GATED_DESCRIPTOR_ONLY_FEATURE_SLOTS: &[(
    BuiltinRenderFeature,
    &str,
    RenderFeatureCapabilityRequirement,
)] = &[(
    BuiltinRenderFeature::SparseTexture,
    "sparse_texture",
    RenderFeatureCapabilityRequirement::SparseTexture,
)];

const ADVANCED_DESCRIPTOR_ONLY_FEATURE_SLOTS: &[(BuiltinRenderFeature, &str, &str)] = &[
    (BuiltinRenderFeature::MeshLod, "mesh_lod", "mesh_lod"),
    (BuiltinRenderFeature::Particle, "particle", "particles"),
    (BuiltinRenderFeature::Terrain, "terrain", "terrain"),
    (BuiltinRenderFeature::Tree, "tree", "tree"),
    (BuiltinRenderFeature::Decal, "decals", "decals"),
    (BuiltinRenderFeature::Projector, "projector", "projector"),
    (BuiltinRenderFeature::Halo, "halo", "halo"),
    (BuiltinRenderFeature::LensFlare, "lens_flare", "lens_flare"),
    (BuiltinRenderFeature::Trail, "trail", "trail"),
    (BuiltinRenderFeature::Billboard, "billboard", "billboard"),
    (BuiltinRenderFeature::Tilemap, "tilemap", "tilemap"),
    (
        BuiltinRenderFeature::TextShaping,
        "text_shaping",
        "text_shaping",
    ),
    (BuiltinRenderFeature::Skybox, "skybox", "skybox"),
    (BuiltinRenderFeature::Cubemap, "cubemap", "cubemap"),
    (
        BuiltinRenderFeature::Texture2dArray,
        "texture_2d_array",
        "texture_2d_array",
    ),
    (BuiltinRenderFeature::NormalMap, "normal_map", "normal_map"),
    (BuiltinRenderFeature::Mipmap, "mipmap", "mipmap"),
    (
        BuiltinRenderFeature::ColorSpace,
        "color_space",
        "color_space",
    ),
];

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
    for (feature, extract_section, requirement) in
        ADVANCED_CAPABILITY_GATED_DESCRIPTOR_ONLY_FEATURE_SLOTS
    {
        let descriptor = feature.descriptor();
        assert_eq!(descriptor.name, *extract_section);
        assert_eq!(
            descriptor.required_extract_sections,
            vec![extract_section.to_string()]
        );
        assert_eq!(descriptor.capability_requirements, vec![*requirement]);
        assert!(descriptor.history_bindings.is_empty());
        assert!(
            descriptor.stage_passes.is_empty(),
            "{feature:?} should stay descriptor-only until its dedicated render plan registers passes"
        );
    }

    for (feature, descriptor_name, extract_section) in ADVANCED_DESCRIPTOR_ONLY_FEATURE_SLOTS {
        let descriptor = feature.descriptor();
        assert_eq!(descriptor.name, *descriptor_name);
        assert_eq!(
            descriptor.required_extract_sections,
            vec![extract_section.to_string()]
        );
        assert!(descriptor.capability_requirements.is_empty());
        assert!(descriptor.history_bindings.is_empty());
        assert!(
            descriptor.stage_passes.is_empty(),
            "{feature:?} should stay descriptor-only until its dedicated render plan registers passes"
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

    assert!(enabled_compiled
        .capability_requirements
        .contains(&RenderFeatureCapabilityRequirement::NeuralCompute));
    assert!(
        !enabled_compiled
            .graph
            .passes()
            .iter()
            .any(|pass| pass.name.contains("neural")),
        "the runtime slot should only declare the capability; plugin descriptors own executable neural passes"
    );
}

#[test]
fn advanced_followup_builtin_slots_compile_only_with_explicit_feature_opt_in() {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    for (feature, _, _) in ADVANCED_CAPABILITY_GATED_DESCRIPTOR_ONLY_FEATURE_SLOTS {
        pipeline
            .renderer
            .features
            .push(RendererFeatureAsset::builtin(*feature));
    }
    for (feature, _, _) in ADVANCED_DESCRIPTOR_ONLY_FEATURE_SLOTS {
        pipeline
            .renderer
            .features
            .push(RendererFeatureAsset::builtin(*feature));
    }

    let default_compiled = pipeline.compile(&test_extract()).unwrap();
    for (feature, extract_section, requirement) in
        ADVANCED_CAPABILITY_GATED_DESCRIPTOR_ONLY_FEATURE_SLOTS
    {
        assert!(
            !default_compiled
                .enabled_features
                .iter()
                .any(|asset| asset.is_builtin(*feature)),
            "{feature:?} should not compile until explicitly opted in"
        );
        assert!(
            !default_compiled
                .required_extract_sections
                .contains(&extract_section.to_string()),
            "{feature:?} should not request extract data until explicitly opted in"
        );
        assert!(
            !default_compiled
                .capability_requirements
                .contains(requirement),
            "{feature:?} should not require backend capability until explicitly opted in"
        );
    }
    for (feature, _, extract_section) in ADVANCED_DESCRIPTOR_ONLY_FEATURE_SLOTS {
        assert!(
            !default_compiled
                .enabled_features
                .iter()
                .any(|asset| asset.is_builtin(*feature)),
            "{feature:?} should not compile until explicitly opted in"
        );
        assert!(
            !default_compiled
                .required_extract_sections
                .contains(&extract_section.to_string()),
            "{feature:?} should not request extract data until explicitly opted in"
        );
    }

    let mut options = RenderPipelineCompileOptions::default();
    for (feature, _, requirement) in ADVANCED_CAPABILITY_GATED_DESCRIPTOR_ONLY_FEATURE_SLOTS {
        options = options
            .with_feature_enabled(*feature)
            .with_capability_enabled(*requirement);
    }
    for (feature, _, _) in ADVANCED_DESCRIPTOR_ONLY_FEATURE_SLOTS {
        options = options.with_feature_enabled(*feature);
    }
    let enabled_compiled = pipeline
        .compile_with_options(&test_extract(), &options)
        .unwrap();

    for (feature, extract_section, requirement) in
        ADVANCED_CAPABILITY_GATED_DESCRIPTOR_ONLY_FEATURE_SLOTS
    {
        assert!(
            enabled_compiled
                .enabled_features
                .iter()
                .any(|asset| asset.is_builtin(*feature)),
            "{feature:?} should compile when explicitly opted in"
        );
        assert!(
            enabled_compiled
                .required_extract_sections
                .contains(&extract_section.to_string()),
            "{feature:?} should reserve its neutral extract section"
        );
        assert!(
            enabled_compiled
                .capability_requirements
                .contains(requirement),
            "{feature:?} should declare its backend capability requirement"
        );
    }
    for (feature, _, extract_section) in ADVANCED_DESCRIPTOR_ONLY_FEATURE_SLOTS {
        assert!(
            enabled_compiled
                .enabled_features
                .iter()
                .any(|asset| asset.is_builtin(*feature)),
            "{feature:?} should compile when explicitly opted in"
        );
        assert!(
            enabled_compiled
                .required_extract_sections
                .contains(&extract_section.to_string()),
            "{feature:?} should reserve its neutral extract section"
        );
    }
    assert_eq!(
        enabled_compiled
            .graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        default_compiled
            .graph
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
            .enabled_features
            .iter()
            .any(|feature| feature.is_builtin(BuiltinRenderFeature::SparseTexture)),
        "feature opt-in without the sparse texture capability should keep the slot out of the graph"
    );
    assert!(!feature_only
        .capability_requirements
        .contains(&RenderFeatureCapabilityRequirement::SparseTexture));

    let capability_enabled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_feature_enabled(BuiltinRenderFeature::SparseTexture)
                .with_capability_enabled(RenderFeatureCapabilityRequirement::SparseTexture),
        )
        .unwrap();
    assert!(capability_enabled
        .enabled_features
        .iter()
        .any(|feature| feature.is_builtin(BuiltinRenderFeature::SparseTexture)));
    assert!(capability_enabled
        .required_extract_sections
        .contains(&"sparse_texture".to_string()));
    assert!(capability_enabled
        .capability_requirements
        .contains(&RenderFeatureCapabilityRequirement::SparseTexture));
    assert!(
        !capability_enabled
            .graph
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
