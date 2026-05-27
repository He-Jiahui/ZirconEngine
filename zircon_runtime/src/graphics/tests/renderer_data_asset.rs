use std::collections::HashMap;
use std::str::FromStr;

use crate::asset::{
    AssetReference, AssetUri, AssetUuid, MaterialAsset, ShaderAsset, ShaderEntryPointAsset,
    ShaderMaterialPropertyAsset, ShaderSourceLanguage, ShaderTextureSlotAsset,
};
use crate::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract,
    RenderMaterialValidationError, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderShaderDefinitionValue, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::Vec4;
use crate::{
    BuiltinRenderFeature, RenderPassStage, RenderPipelineAsset, RenderPipelineAssetContext,
    RenderPipelineCompileOptions, RendererDataDocument, RendererDataDocumentError,
    RendererFeatureContractDiagnostic,
};

#[test]
fn renderer_data_document_toml_roundtrip_preserves_srp_fields() {
    let document = RendererDataDocument::from_toml_str(SAMPLE_RENDERER_DATA).unwrap();

    assert_eq!(document.version, 1);
    assert_eq!(document.name, "forward-contract");
    assert_eq!(document.stages, vec!["DepthPrepass", "Opaque3d", "Ui"]);
    assert_eq!(document.features.len(), 2);
    assert_eq!(document.features[0].source, "Mesh");
    assert_eq!(document.features[0].quality_gate.as_deref(), Some("Mesh"));
    assert_eq!(
        document.features[0].shader,
        Some(asset_reference(
            "00000000-0000-0000-0000-000000000011",
            "res://shaders/mesh.zshader"
        ))
    );
    assert_eq!(
        document.features[0].material,
        Some(asset_reference(
            "00000000-0000-0000-0000-000000000012",
            "res://materials/mesh.zmaterial"
        ))
    );
    assert_eq!(
        document.features[0].required_entry_points,
        vec!["vs_main", "fs_main"]
    );
    assert_eq!(document.features[0].expected_properties, vec!["base_color"]);
    assert_eq!(
        document.features[0].expected_texture_slots,
        vec!["base_color"]
    );
    assert_eq!(
        document.features[0].local_config.get("variant").unwrap(),
        "lit"
    );

    let encoded = document.to_toml_string().unwrap();
    let decoded = RendererDataDocument::from_toml_str(&encoded).unwrap();

    assert_eq!(decoded, document);
}

#[test]
fn renderer_data_document_converts_to_renderer_asset() {
    let renderer = RendererDataDocument::from_toml_str(SAMPLE_RENDERER_DATA)
        .unwrap()
        .to_renderer_asset()
        .unwrap();

    assert_eq!(renderer.name, "forward-contract");
    assert_eq!(
        renderer.stages,
        vec![
            RenderPassStage::DepthPrepass,
            RenderPassStage::Opaque3d,
            RenderPassStage::Ui,
        ]
    );
    assert!(renderer.features[0].is_builtin(BuiltinRenderFeature::Mesh));
    assert!(renderer.features[0].enabled);
    assert_eq!(
        renderer.features[0].quality_gate,
        Some(BuiltinRenderFeature::Mesh)
    );
    assert_eq!(
        renderer.features[0].asset_references.shader,
        Some(asset_reference(
            "00000000-0000-0000-0000-000000000011",
            "res://shaders/mesh.zshader"
        ))
    );
    assert_eq!(
        renderer.features[0].asset_references.material,
        Some(asset_reference(
            "00000000-0000-0000-0000-000000000012",
            "res://materials/mesh.zmaterial"
        ))
    );
    assert_eq!(
        renderer.features[0].asset_references.required_entry_points,
        vec!["vs_main", "fs_main"]
    );
    assert_eq!(
        renderer.features[0].asset_references.expected_properties,
        vec!["base_color"]
    );
    assert_eq!(
        renderer.features[0].asset_references.expected_texture_slots,
        vec!["base_color"]
    );
    assert_eq!(
        renderer.features[0].local_config.get("variant").unwrap(),
        "lit"
    );
}

#[test]
fn renderer_data_document_preserves_disabled_features() {
    let renderer = RendererDataDocument::from_toml_str(SAMPLE_RENDERER_DATA)
        .unwrap()
        .to_renderer_asset()
        .unwrap();

    let ui = &renderer.features[1];
    assert!(ui.is_builtin(BuiltinRenderFeature::Ui));
    assert!(!ui.enabled);
    assert_eq!(ui.quality_gate, None);
}

#[test]
fn renderer_feature_asset_builders_preserve_shader_material_contract_references() {
    let shader = asset_reference("custom-shader", "res://shaders/custom.zshader");
    let material = asset_reference("custom-material", "res://materials/custom.zmaterial");

    let feature = crate::RendererFeatureAsset::builtin(BuiltinRenderFeature::PostProcess)
        .with_shader_reference(shader.clone())
        .with_material_reference(material.clone())
        .with_required_entry_point("fullscreen_fs")
        .with_expected_property("exposure")
        .with_expected_texture_slot("source_color");

    assert_eq!(feature.asset_references.shader, Some(shader));
    assert_eq!(feature.asset_references.material, Some(material));
    assert_eq!(
        feature.asset_references.required_entry_points,
        vec!["fullscreen_fs"]
    );
    assert_eq!(
        feature.asset_references.expected_properties,
        vec!["exposure"]
    );
    assert_eq!(
        feature.asset_references.expected_texture_slots,
        vec!["source_color"]
    );
}

#[test]
fn renderer_data_document_rejects_unknown_stage_names() {
    let error = RendererDataDocument::from_toml_str(
        r#"
version = 1
name = "bad-stage"
stages = ["Opaque"]
"#,
    )
    .unwrap()
    .to_renderer_asset()
    .unwrap_err();

    assert_eq!(
        error,
        RendererDataDocumentError::UnknownRenderPassStage {
            value: "Opaque".to_string(),
        }
    );
}

#[test]
fn renderer_data_document_rejects_unknown_feature_sources() {
    let error = RendererDataDocument::from_toml_str(
        r#"
version = 1
name = "bad-feature"
stages = ["Opaque3d"]

[[features]]
name = "Unknown"
source = "Unknown"
enabled = true
"#,
    )
    .unwrap()
    .to_renderer_asset()
    .unwrap_err();

    assert_eq!(
        error,
        RendererDataDocumentError::UnknownRenderFeatureSource {
            value: "Unknown".to_string(),
        }
    );
}

#[test]
fn asset_aware_compile_reports_missing_shader_and_material_without_blocking_graph() {
    let shader = asset_reference("missing-shader", "res://shaders/missing.zshader");
    let material = asset_reference("missing-material", "res://materials/missing.zmaterial");
    let pipeline = pipeline_with_mesh_feature(
        crate::RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_shader_reference(shader.clone())
            .with_material_reference(material.clone()),
    );

    let report = pipeline
        .compile_with_asset_context(
            &test_extract(),
            &RenderPipelineCompileOptions::default(),
            &InMemoryRenderPipelineAssetContext::default(),
        )
        .unwrap();

    assert!(report
        .pipeline
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "opaque-mesh"));
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::ShaderMissing { feature, reference }
            if feature == "mesh" && reference == &shader
    )));
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialMissing { feature, reference }
            if feature == "mesh" && reference == &material
    )));
}

#[test]
fn asset_aware_compile_reports_shader_contract_expectation_gaps() {
    let shader = asset_reference("mesh-shader", "res://shaders/mesh.zshader");
    let pipeline = pipeline_with_mesh_feature(
        crate::RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_shader_reference(shader.clone())
            .with_required_entry_point("missing_vs")
            .with_expected_property("roughness")
            .with_expected_texture_slot("normal"),
    );
    let context = InMemoryRenderPipelineAssetContext::default().with_shader(
        shader.clone(),
        shader_with_validation_diagnostic("capture missing"),
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
        RendererFeatureContractDiagnostic::MissingEntryPoint { feature, shader: diagnostic_shader, entry_point }
            if feature == "mesh" && diagnostic_shader == &shader && entry_point == "missing_vs"
    )));
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MissingProperty { feature, shader: diagnostic_shader, property }
            if feature == "mesh" && diagnostic_shader == &shader && property == "roughness"
    )));
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MissingTextureSlot { feature, shader: diagnostic_shader, slot }
            if feature == "mesh" && diagnostic_shader == &shader && slot == "normal"
    )));
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::ShaderValidation { feature, shader: diagnostic_shader, diagnostic }
            if feature == "mesh" && diagnostic_shader == &shader && diagnostic == "capture missing"
    )));
}

#[test]
fn asset_aware_compile_reports_shader_payload_readiness_gaps() {
    let shader = asset_reference("readiness-shader", "res://shaders/readiness.zshader");
    let pipeline = pipeline_with_mesh_feature(
        crate::RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_shader_reference(shader.clone()),
    );
    let mut shader_asset = shader_contract();
    shader_asset.uri = shader.locator.clone();
    shader_asset.source_language = ShaderSourceLanguage::Glsl;
    shader_asset.source = "void main() {}".to_string();
    shader_asset.wgsl_source.clear();
    shader_asset.entry_points.push(ShaderEntryPointAsset {
        name: "pixel_main".to_string(),
        stage: "pixel".to_string(),
    });
    shader_asset.shader_defs = vec![
        RenderShaderDefinitionValue::from("USE_FOG"),
        RenderShaderDefinitionValue::from(" "),
        RenderShaderDefinitionValue::bool(" USE_FOG ", false),
    ];
    let context =
        InMemoryRenderPipelineAssetContext::default().with_shader(shader.clone(), shader_asset);

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
            shader: diagnostic_shader,
            diagnostic,
        } if feature == "mesh"
            && diagnostic_shader == &shader
            && diagnostic.contains("does not provide emitted WGSL")
    )));
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::ShaderValidation {
            feature,
            shader: diagnostic_shader,
            diagnostic,
        } if feature == "mesh"
            && diagnostic_shader == &shader
            && diagnostic.contains("unsupported stage `pixel`")
    )));
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::ShaderValidation {
            feature,
            shader: diagnostic_shader,
            diagnostic,
        } if feature == "mesh"
            && diagnostic_shader == &shader
            && diagnostic.contains("empty after trimming")
    )));
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::ShaderValidation {
            feature,
            shader: diagnostic_shader,
            diagnostic,
        } if feature == "mesh"
            && diagnostic_shader == &shader
            && diagnostic.contains("duplicated")
    )));
}

#[test]
fn asset_aware_compile_reports_material_contract_diagnostics() {
    let feature_shader = asset_reference("feature-shader", "res://shaders/feature.zshader");
    let material_shader = asset_reference("material-shader", "res://shaders/material.zshader");
    let material = asset_reference("material", "res://materials/mismatch.zmaterial");
    let pipeline = pipeline_with_mesh_feature(
        crate::RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_shader_reference(feature_shader.clone())
            .with_material_reference(material.clone()),
    );
    let context = InMemoryRenderPipelineAssetContext::default()
        .with_shader(feature_shader.clone(), shader_contract())
        .with_material(
            material,
            material_with_contract_gaps(material_shader.clone()),
        );

    let report = pipeline
        .compile_with_asset_context(
            &test_extract(),
            &RenderPipelineCompileOptions::default(),
            &context,
        )
        .unwrap();

    assert!(report
        .pipeline
        .graph
        .passes()
        .iter()
        .any(|pass| pass.name == "opaque-mesh"));
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialShaderMismatch {
            feature,
            feature_shader: diagnostic_feature_shader,
            material_shader: diagnostic_material_shader,
        } if feature == "mesh"
            && diagnostic_feature_shader == &feature_shader
            && diagnostic_material_shader == &material_shader
    )));
    assert_material_validation(&report.diagnostics, |error| {
        matches!(
            error,
            RenderMaterialValidationError::UnknownPropertyOverride { name, .. } if name == "unknown_scalar"
        )
    });
    assert_material_validation(&report.diagnostics, |error| {
        matches!(
            error,
            RenderMaterialValidationError::PropertyOverrideTypeMismatch { name, expected, .. }
                if name == "base_color" && expected == "vec4"
        )
    });
    assert_material_validation(&report.diagnostics, |error| {
        matches!(
            error,
            RenderMaterialValidationError::MissingRequiredProperty { name, .. } if name == "emissive_power"
        )
    });
    assert_material_validation(&report.diagnostics, |error| {
        matches!(
            error,
            RenderMaterialValidationError::UnknownTextureSlot { slot, .. } if slot == "unknown_slot"
        )
    });
}

#[test]
fn asset_aware_compile_reports_material_local_validation_diagnostics() {
    let feature_shader = asset_reference("feature-shader", "res://shaders/feature.zshader");
    let material_reference = asset_reference("material", "res://materials/local-errors.zmaterial");
    let pipeline = pipeline_with_mesh_feature(
        crate::RendererFeatureAsset::builtin(BuiltinRenderFeature::Mesh)
            .with_shader_reference(feature_shader.clone())
            .with_material_reference(material_reference.clone()),
    );
    let mut material = material_with_contract_gaps(feature_shader.clone());
    material.alpha_mode = crate::asset::AlphaMode::Mask { cutoff: 2.0 };
    material
        .validation_diagnostics
        .push("material importer note".to_string());
    let context = InMemoryRenderPipelineAssetContext::default()
        .with_shader(feature_shader, shader_contract())
        .with_material(material_reference.clone(), material);

    let report = pipeline
        .compile_with_asset_context(
            &test_extract(),
            &RenderPipelineCompileOptions::default(),
            &context,
        )
        .unwrap();

    assert_material_validation(&report.diagnostics, |error| {
        matches!(
            error,
            RenderMaterialValidationError::InvalidMaskCutoff { cutoff } if *cutoff == 2.0
        )
    });
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialDiagnostic { feature, material, diagnostic }
            if feature == "mesh"
                && material == &material_reference
                && diagnostic == "material importer note"
    )));
}

const SAMPLE_RENDERER_DATA: &str = r#"
version = 1
name = "forward-contract"
stages = ["DepthPrepass", "Opaque3d", "Ui"]

[[features]]
name = "Mesh"
source = "Mesh"
enabled = true
quality_gate = "Mesh"
required_entry_points = ["vs_main", "fs_main"]
expected_properties = ["base_color"]
expected_texture_slots = ["base_color"]
local_config = { variant = "lit" }

[features.shader]
uuid = "00000000-0000-0000-0000-000000000011"
url = "res://shaders/mesh.zshader"

[features.material]
uuid = "00000000-0000-0000-0000-000000000012"
url = "res://materials/mesh.zmaterial"

[[features]]
name = "Ui"
source = "Ui"
enabled = false
"#;

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

fn pipeline_with_mesh_feature(feature: crate::RendererFeatureAsset) -> RenderPipelineAsset {
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

fn shader_with_validation_diagnostic(diagnostic: &str) -> ShaderAsset {
    let mut shader = shader_contract();
    shader.validation_diagnostics.push(diagnostic.to_string());
    shader
}

fn shader_contract() -> ShaderAsset {
    ShaderAsset {
        uri: AssetUri::parse("res://shaders/feature.zshader").unwrap(),
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
name = "Mismatch"

[shader]
uuid = "00000000-0000-0000-0000-000000000099"
url = "res://shaders/material.zshader"

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
    material.shader = shader;
    material
}

fn assert_material_validation(
    diagnostics: &[RendererFeatureContractDiagnostic],
    predicate: impl Fn(&RenderMaterialValidationError) -> bool,
) {
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RendererFeatureContractDiagnostic::MaterialValidation { feature, error }
            if feature == "mesh" && predicate(error)
    )));
}
