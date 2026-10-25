use std::collections::BTreeMap;

use super::{
    compile_material_graph_with_diagnostics, compile_material_graph_with_shader_contract,
    MaterialGraphShaderCompileError,
};
use zircon_runtime::asset::assets::generate_material_artifact;
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialGraphAsset, MaterialGraphLinkAsset, MaterialGraphNodeAsset,
    MaterialGraphNodeKindAsset as Node, ShaderAsset, ShaderMaterialPropertyAsset,
    ShaderSourceLanguage, ShaderTextureSlotAsset,
};
use zircon_runtime::core::framework::render::{
    MaterialPropertyKind, RenderMaterialDiagnosticSource, RenderMaterialValidationError,
    ShaderAssetKind,
};

#[test]
fn material_graph_shader_contract_preserves_compiled_material_and_original_sources() {
    let graph = graph();
    let shader = shader();
    let before_graph = graph.clone();
    let before_shader = shader.clone();

    let material = compile_material_graph_with_shader_contract(&graph, &shader).unwrap();
    assert_eq!(
        material,
        compile_material_graph_with_diagnostics(&graph).unwrap()
    );
    assert_eq!(material.shader, graph.shader.clone().unwrap());
    assert_eq!(
        material.standard_material_descriptor().base_color,
        [0.2, 0.4, 0.8, 1.0]
    );
    assert_eq!(material.standard_material_descriptor().roughness, 0.35);
    assert_eq!(graph, before_graph);
    assert_eq!(shader, before_shader);
}

#[test]
fn material_graph_shader_contract_keeps_graph_input_errors_in_graph_coordinates() {
    let mut graph = graph();
    graph.links[0].from_pin = "missing".into();
    let mut shader = shader();
    shader.uri = uri("res://shaders/another.zshader");
    let before = graph.clone();

    let MaterialGraphShaderCompileError::Graph(diagnostics) =
        compile_material_graph_with_shader_contract(&graph, &shader).unwrap_err()
    else {
        panic!("invalid graph must retain its own diagnostics before shader admission");
    };
    assert_eq!(
        diagnostics,
        compile_material_graph_with_diagnostics(&graph).unwrap_err()
    );
    assert_eq!(graph, before);
}

#[test]
fn material_graph_shader_contract_rejects_another_shader_without_rebinding_the_reference() {
    let graph = graph();
    let mut shader = shader();
    shader.uri = uri("res://shaders/另一份很长的材质着色器.zshader");
    let before_graph = graph.clone();
    let before_shader = shader.clone();

    let error = compile_material_graph_with_shader_contract(&graph, &shader).unwrap_err();
    assert_eq!(
        error,
        MaterialGraphShaderCompileError::ShaderLocatorMismatch {
            expected: graph.shader.clone().unwrap(),
            actual: shader.uri.clone(),
        }
    );
    assert_eq!(graph, before_graph);
    assert_eq!(shader, before_shader);
}

#[test]
fn material_graph_shader_contract_uses_runtime_surface_kind_validation() {
    let graph = graph();
    let mut shader = shader();
    shader.kind = ShaderAssetKind::Compute;

    let MaterialGraphShaderCompileError::ShaderContract(diagnostics) =
        compile_material_graph_with_shader_contract(&graph, &shader).unwrap_err()
    else {
        panic!("compute shaders cannot be admitted as material surface shaders");
    };
    assert!(matches!(
        diagnostics.as_slice(),
        [RenderMaterialValidationError::ShaderReadinessDiagnostic {
            source: RenderMaterialDiagnosticSource::ShaderReadiness,
            path,
            ..
        }] if path == "shader.kind"
    ));
}

#[test]
fn material_graph_shader_contract_preserves_multiple_runtime_paths_and_sources() {
    let graph = graph();
    let mut shader = shader();
    shader.property_schema.push(ShaderMaterialPropertyAsset {
        name: "detail_strength".into(),
        kind: MaterialPropertyKind::Float,
        required: true,
        default: None,
        editor: BTreeMap::new(),
    });
    shader.texture_slots.push(ShaderTextureSlotAsset {
        name: "detail_mask".into(),
        kind: "texture2d".into(),
        required: true,
        default: None,
        sampler: None,
        group: None,
        label: None,
        option: None,
        st: false,
        editor: BTreeMap::new(),
    });
    let generated = generate_material_artifact(
        &shader.property_schema,
        &shader.options,
        &shader.texture_slots,
    );
    shader.material_property_layout = generated.property_layout;
    shader.material_option_table = generated.option_table;
    shader.generated_material_wgsl = generated.wgsl_source;
    let before = shader.clone();

    let MaterialGraphShaderCompileError::ShaderContract(diagnostics) =
        compile_material_graph_with_shader_contract(&graph, &shader).unwrap_err()
    else {
        panic!("required shader fields cannot be silently replaced by graph defaults");
    };
    assert_eq!(diagnostics.len(), 2);
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RenderMaterialValidationError::MissingRequiredProperty {
            source: RenderMaterialDiagnosticSource::ShaderSchema,
            path,
            name,
        } if path == "overrides.detail_strength" && name == "detail_strength"
    )));
    assert!(diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        RenderMaterialValidationError::MissingRequiredTextureSlot {
            source: RenderMaterialDiagnosticSource::ShaderSchema,
            path,
            slot,
        } if path == "textures.detail_mask" && slot == "detail_mask"
    )));
    assert_eq!(shader, before);
}

#[test]
fn material_graph_shader_contract_preserves_existing_import_diagnostics() {
    let graph = graph();
    let mut shader = shader();
    shader.validation_diagnostics = vec![
        "properties.base_color 的真实导入诊断".into(),
        "texture binding detail_mask: incompatible capture".into(),
    ];
    let before = shader.clone();

    assert_eq!(
        compile_material_graph_with_shader_contract(&graph, &shader).unwrap_err(),
        MaterialGraphShaderCompileError::ShaderImportDiagnostics(
            shader.validation_diagnostics.clone()
        )
    );
    assert_eq!(shader, before);
}

fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).unwrap()
}

fn graph() -> MaterialGraphAsset {
    MaterialGraphAsset {
        uri: uri("res://materials/graph.toml"),
        name: "Shader contract authoring fixture".into(),
        shader: Some(AssetReference::from_locator(uri(
            "res://shaders/pbr.zshader",
        ))),
        nodes: vec![
            MaterialGraphNodeAsset {
                id: "base".into(),
                position: [0.0, 0.0],
                kind: Node::VectorParameter {
                    name: "Base Color".into(),
                    default: [0.2, 0.4, 0.8, 1.0],
                },
            },
            MaterialGraphNodeAsset {
                id: "roughness".into(),
                position: [0.0, 160.0],
                kind: Node::ScalarParameter {
                    name: "Roughness".into(),
                    default: 0.35,
                },
            },
            MaterialGraphNodeAsset {
                id: "output".into(),
                position: [320.0, 0.0],
                kind: Node::Output,
            },
        ],
        links: vec![
            MaterialGraphLinkAsset {
                from_node: "base".into(),
                from_pin: "value".into(),
                to_node: "output".into(),
                to_pin: "base_color".into(),
            },
            MaterialGraphLinkAsset {
                from_node: "roughness".into(),
                from_pin: "value".into(),
                to_node: "output".into(),
                to_pin: "roughness".into(),
            },
        ],
        parameters: BTreeMap::new(),
    }
}

// A domain unit fixture for Runtime contract checks, not a ready GPU shader,
// published resource, preview producer or substitute for engine admission.
fn shader() -> ShaderAsset {
    ShaderAsset {
        uri: uri("res://shaders/pbr.zshader"),
        kind: ShaderAssetKind::Surface,
        source_language: ShaderSourceLanguage::Wgsl,
        source: String::new(),
        wgsl_source: String::new(),
        import_path: None,
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema: Vec::new(),
        options: Vec::new(),
        texture_slots: Vec::new(),
        shading_model: None,
        render_state: Default::default(),
        queue: None,
        disabled_passes: Vec::new(),
        resources: Vec::new(),
        material_property_layout: Default::default(),
        material_option_table: Default::default(),
        generated_material_wgsl: String::new(),
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}
