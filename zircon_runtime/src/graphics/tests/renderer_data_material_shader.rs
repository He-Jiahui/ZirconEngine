use std::collections::HashMap;
use std::str::FromStr;

use crate::asset::{
    AlphaMode, AssetReference, AssetUri, AssetUuid, MaterialAsset, ShaderAsset,
    ShaderEntryPointAsset, ShaderMaterialPropertyAsset, ShaderSourceLanguage,
    ShaderTextureSlotAsset,
};
use crate::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract,
    RenderMaterialValidationError, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::Vec4;
use crate::{
    BuiltinRenderFeature, RenderPipelineAsset, RenderPipelineAssetContext,
    RenderPipelineCompileOptions, RendererFeatureAsset, RendererFeatureContractDiagnostic,
};

#[test]
fn asset_aware_compile_uses_material_shader_for_material_only_contract_diagnostics() {
    let material_reference =
        asset_reference("material-only", "res://materials/material-only.zmaterial");
    let material_shader = asset_reference("material-shader", "res://shaders/material-only.zshader");
    let pipeline = pipeline_with_mesh_feature(
        RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_material_reference(material_reference.clone()),
    );
    let context = InMemoryRenderPipelineAssetContext::default()
        .with_material(
            material_reference.clone(),
            material_with_contract_gaps(material_shader.clone()),
        )
        .with_shader(material_shader.clone(), shader_contract());

    let report = pipeline
        .compile_with_asset_context(
            &test_extract(),
            &RenderPipelineCompileOptions::default(),
            &context,
        )
        .unwrap();

    assert_material_validation_with_shader(
        &report.diagnostics,
        &material_reference,
        &material_shader,
        |error| {
            matches!(
                error,
                RenderMaterialValidationError::UnknownPropertyOverride { name, .. }
                    if name == "unknown_scalar"
            )
        },
    );
    assert_material_validation_with_shader(
        &report.diagnostics,
        &material_reference,
        &material_shader,
        |error| {
            matches!(
                error,
                RenderMaterialValidationError::MissingRequiredProperty { name, .. }
                    if name == "emissive_power"
            )
        },
    );
    assert_material_validation_with_shader(
        &report.diagnostics,
        &material_reference,
        &material_shader,
        |error| {
            matches!(
                error,
                RenderMaterialValidationError::UnknownTextureSlot { slot, .. }
                    if slot == "unknown_slot"
            )
        },
    );
    assert_material_validation_without_shader(&report.diagnostics, &material_reference, |error| {
        matches!(
            error,
            RenderMaterialValidationError::InvalidMaskCutoff { cutoff } if *cutoff == 2.0
        )
    });
    assert!(!report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialShaderMismatch { .. }
    )));
}

#[test]
fn asset_aware_compile_reports_material_owned_shader_readiness_diagnostics() {
    let material_reference =
        asset_reference("material-readiness", "res://materials/readiness.zmaterial");
    let material_shader = asset_reference(
        "material-readiness-shader",
        "res://shaders/readiness.zshader",
    );
    let pipeline = pipeline_with_mesh_feature(
        RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_material_reference(material_reference.clone()),
    );
    let mut shader = shader_contract();
    shader
        .validation_diagnostics
        .push("material shader readiness warning".to_string());
    let context = InMemoryRenderPipelineAssetContext::default()
        .with_material(
            material_reference,
            material_with_contract_gaps(material_shader.clone()),
        )
        .with_shader(material_shader.clone(), shader);

    let report = pipeline
        .compile_with_asset_context(
            &test_extract(),
            &RenderPipelineCompileOptions::default(),
            &context,
        )
        .unwrap();

    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::ShaderValidation {
            feature,
            shader,
            diagnostic,
        } if feature == "mesh"
            && shader == &material_shader
            && diagnostic == "material shader readiness warning"
    )));
}

#[test]
fn asset_aware_compile_reports_material_owned_shader_missing() {
    let material_reference = asset_reference(
        "missing-material-shader",
        "res://materials/missing-shader.zmaterial",
    );
    let material_shader = asset_reference("missing-shader", "res://shaders/missing.zshader");
    let pipeline = pipeline_with_mesh_feature(
        RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_material_reference(material_reference.clone()),
    );
    let context = InMemoryRenderPipelineAssetContext::default().with_material(
        material_reference.clone(),
        material_with_contract_gaps(material_shader.clone()),
    );

    let report = pipeline
        .compile_with_asset_context(
            &test_extract(),
            &RenderPipelineCompileOptions::default(),
            &context,
        )
        .unwrap();

    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialShaderMissing { feature, material, shader }
            if feature == "mesh" && material == &material_reference && shader == &material_shader
    )));
}

fn asset_reference(label: &str, uri: &str) -> AssetReference {
    AssetReference::new(
        AssetUuid::from_str(label).unwrap_or_else(|_| AssetUuid::from_stable_label(label)),
        AssetUri::parse(uri).unwrap(),
    )
}

#[derive(Default)]
struct InMemoryRenderPipelineAssetContext {
    shaders: HashMap<AssetReference, ShaderAsset>,
    materials: HashMap<AssetReference, MaterialAsset>,
}

impl InMemoryRenderPipelineAssetContext {
    fn with_shader(mut self, reference: AssetReference, shader: ShaderAsset) -> Self {
        self.shaders.insert(reference, shader);
        self
    }

    fn with_material(mut self, reference: AssetReference, material: MaterialAsset) -> Self {
        self.materials.insert(reference, material);
        self
    }
}

impl RenderPipelineAssetContext for InMemoryRenderPipelineAssetContext {
    fn load_shader_asset(&self, reference: &AssetReference) -> Option<ShaderAsset> {
        self.shaders.get(reference).cloned()
    }

    fn load_material_asset(&self, reference: &AssetReference) -> Option<MaterialAsset> {
        self.materials.get(reference).cloned()
    }
}

fn pipeline_with_mesh_feature(feature: RendererFeatureAsset) -> RenderPipelineAsset {
    let mut pipeline = RenderPipelineAsset::default_forward_plus();
    let mesh = pipeline
        .renderer
        .features
        .iter_mut()
        .find(|feature| feature.is_builtin(BuiltinRenderFeature::Mesh))
        .unwrap();
    *mesh = feature;
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

fn shader_contract() -> ShaderAsset {
    ShaderAsset {
        uri: AssetUri::parse("res://shaders/material-only.zshader").unwrap(),
        source_language: ShaderSourceLanguage::Wgsl,
        source: String::new(),
        wgsl_source:
            "@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0); }"
                .to_string(),
        import_path: None,
        entry_points: vec![ShaderEntryPointAsset {
            name: "vs_main".to_string(),
            stage: "vertex".to_string(),
        }],
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema: vec![
            ShaderMaterialPropertyAsset {
                name: "base_color".to_string(),
                kind: "vec4".to_string(),
                required: true,
                default: None,
                editor: Default::default(),
            },
            ShaderMaterialPropertyAsset {
                name: "emissive_power".to_string(),
                kind: "float".to_string(),
                required: true,
                default: None,
                editor: Default::default(),
            },
        ],
        texture_slots: vec![ShaderTextureSlotAsset {
            name: "base_color".to_string(),
            kind: "texture2d".to_string(),
            required: false,
            default: Some("white".to_string()),
            sampler: Some("linear_repeat".to_string()),
            group: Some("Surface".to_string()),
            label: Some("Base Color".to_string()),
            editor: Default::default(),
        }],
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn material_with_contract_gaps(shader: AssetReference) -> MaterialAsset {
    let mut material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "MaterialOnly"

[shader]
uuid = "00000000-0000-0000-0000-000000000099"
url = "res://shaders/material-only.zshader"

[overrides]
base_color = true
unknown_scalar = 3.0

[textures.base_color]
fallback = "white"

[textures.unknown_slot]
uuid = "00000000-0000-0000-0000-000000000098"
url = "res://textures/extra.png"
"#,
    )
    .unwrap();
    material.alpha_mode = AlphaMode::Mask { cutoff: 2.0 };
    material.shader = shader;
    material
}

fn assert_material_validation_with_shader(
    diagnostics: &[RendererFeatureContractDiagnostic],
    expected_material: &AssetReference,
    expected_shader: &AssetReference,
    predicate: impl Fn(&RenderMaterialValidationError) -> bool,
) {
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialValidation {
            feature,
            material,
            shader: Some(shader),
            error,
        }
            if feature == "mesh"
                && material == expected_material
                && shader == expected_shader
                && predicate(error)
    )));
}

fn assert_material_validation_without_shader(
    diagnostics: &[RendererFeatureContractDiagnostic],
    expected_material: &AssetReference,
    predicate: impl Fn(&RenderMaterialValidationError) -> bool,
) {
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialValidation {
            feature,
            material,
            shader: None,
            error,
        }
            if feature == "mesh" && material == expected_material && predicate(error)
    )));
}
