use std::collections::BTreeMap;

use super::*;
use zircon_editor::core::editor_extension::EditorExtensionRegistry;
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_editor::EditorPlugin;
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialGraphAsset, MaterialGraphLinkAsset, MaterialGraphNodeAsset,
    MaterialGraphNodeKindAsset, MaterialGraphParameterAsset,
};
use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::plugin::{ExportPackagingStrategy, PluginModuleKind};

#[test]
fn material_authoring_registration_exposes_menu_items_and_payload_schemas() {
    let mut registry = EditorExtensionRegistry::default();
    editor_plugin()
        .register_editor_extensions(&mut registry)
        .expect("material authoring registration");
    let operation =
        EditorOperationPath::parse("material_editor.graph.compile").expect("valid material path");
    let descriptor = registry
        .operations()
        .descriptor(&operation)
        .expect("compile operation registered");

    assert_eq!(
        descriptor.menu_path(),
        Some("Plugins/Material Editor/Compile Graph")
    );
    assert_eq!(
        descriptor.payload_schema_id(),
        Some("material_editor.compile_graph.v1")
    );
    assert!(registry.menu_items().iter().any(|item| {
        item.path() == "Plugins/Material Editor/Compile Graph" && item.operation() == &operation
    }));
}

#[test]
fn material_editor_package_manifest_declares_editor_only_metadata() {
    let manifest = package_manifest();
    let editor_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == zircon_runtime::plugin::PluginModuleKind::Editor)
        .expect("material editor module");

    assert_eq!(manifest.category, "authoring");
    assert_eq!(
        manifest.supported_targets,
        vec![zircon_runtime::builtin::RuntimeTargetMode::EditorHost]
    );
    assert_eq!(manifest.capabilities, vec![CAPABILITY.to_string()]);
    assert_eq!(editor_module.capabilities, manifest.capabilities);
}

#[test]
fn material_editor_package_manifest_declares_editor_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest
        .default_packaging
        .contains(&ExportPackagingStrategy::NativeDynamic));
    let distribution = manifest
        .distribution
        .as_ref()
        .expect("material_editor declares standalone distribution");
    assert_eq!(distribution.forms, vec!["dist"]);
    assert_eq!(
        distribution.default_packaging,
        vec![ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.dist_crate, MATERIAL_EDITOR_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert!(distribution.runtime_entry.is_empty());
    assert_eq!(distribution.editor_entry, MATERIAL_EDITOR_DIST_EDITOR_ENTRY);

    let dist_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "material_editor.dist")
        .expect("material_editor dist module is declared");
    assert_eq!(dist_module.kind, PluginModuleKind::Native);
    assert_eq!(dist_module.crate_name, MATERIAL_EDITOR_DIST_CRATE_NAME);
    assert_eq!(
        dist_module.target_modes,
        vec![RuntimeTargetMode::EditorHost]
    );
    assert_eq!(dist_module.capabilities, vec![CAPABILITY.to_string()]);
}

#[test]
fn material_graph_compile_writes_minimal_material_asset_contract() {
    let mut graph = graph_with_shader();
    graph.nodes.push(node(
        "base",
        MaterialGraphNodeKindAsset::VectorParameter {
            name: "base_color".to_string(),
            default: [0.25, 0.5, 0.75, 1.0],
        },
    ));
    graph
        .nodes
        .push(node("output", MaterialGraphNodeKindAsset::Output));
    graph
        .links
        .push(link("base", "value", "output", "base_color"));

    let material = compile_material_graph(&graph).expect("valid material graph compiles");

    assert_eq!(material.name.as_deref(), Some("Test Material"));
    assert_eq!(material.base_color, [0.25, 0.5, 0.75, 1.0]);
    assert!(material.parent.is_none());
    assert!(material.options.is_empty());
    assert!(material.queue.is_none());
    assert!(material.base_color_texture.is_none());
}

#[test]
fn material_graph_compile_uses_parameter_defaults_and_math_nodes() {
    let mut graph = graph_with_shader();
    graph.parameters.insert(
        "tint".to_string(),
        MaterialGraphParameterAsset::Vector([0.2, 0.3, 0.4, 1.0]),
    );
    graph.nodes.push(node(
        "tint",
        MaterialGraphNodeKindAsset::VectorParameter {
            name: "tint".to_string(),
            default: [1.0, 1.0, 1.0, 1.0],
        },
    ));
    graph.nodes.push(node(
        "gain",
        MaterialGraphNodeKindAsset::ScalarParameter {
            name: "gain".to_string(),
            default: 2.0,
        },
    ));
    graph
        .nodes
        .push(node("multiply", MaterialGraphNodeKindAsset::Multiply));
    graph
        .nodes
        .push(node("output", MaterialGraphNodeKindAsset::Output));
    graph.links.push(link("tint", "value", "multiply", "a"));
    graph.links.push(link("gain", "value", "multiply", "b"));
    graph
        .links
        .push(link("multiply", "value", "output", "base_color"));

    let material = compile_material_graph(&graph).expect("valid math graph compiles");

    assert_eq!(material.base_color, [0.4, 0.6, 0.8, 1.0]);
}

#[test]
fn material_graph_compile_maps_texture_sample_to_base_color_texture() {
    let mut graph = graph_with_shader();
    let texture = asset_ref("res://textures/albedo.png");
    graph.nodes.push(node(
        "albedo",
        MaterialGraphNodeKindAsset::TextureSample {
            texture: texture.clone(),
        },
    ));
    graph
        .nodes
        .push(node("output", MaterialGraphNodeKindAsset::Output));
    graph
        .links
        .push(link("albedo", "color", "output", "base_color"));

    let material = compile_material_graph(&graph).expect("valid texture graph compiles");

    assert_eq!(material.base_color_texture, Some(texture));
    assert_eq!(material.base_color, [1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn material_graph_validation_reports_missing_output_duplicate_node_and_required_input() {
    let mut graph = graph_with_shader();
    graph
        .nodes
        .push(node("color", MaterialGraphNodeKindAsset::Add));
    graph
        .nodes
        .push(node("color", MaterialGraphNodeKindAsset::Multiply));

    let diagnostics = validate_material_graph(&graph);

    assert!(diagnostics
        .iter()
        .any(|message| message.contains("duplicate node `color`")));
    assert!(diagnostics
        .iter()
        .any(|message| message.contains("has no output node")));
    assert!(diagnostics
        .iter()
        .any(|message| message.contains("missing required input `a`")));
}

#[test]
fn material_graph_compile_requires_shader_target() {
    let mut graph = graph_with_shader();
    graph.shader = None;
    graph.nodes.push(node(
        "base",
        MaterialGraphNodeKindAsset::VectorParameter {
            name: "base_color".to_string(),
            default: [1.0, 1.0, 1.0, 1.0],
        },
    ));
    graph
        .nodes
        .push(node("output", MaterialGraphNodeKindAsset::Output));
    graph
        .links
        .push(link("base", "value", "output", "base_color"));

    let diagnostics = compile_material_graph(&graph).expect_err("shader is required");

    assert!(diagnostics
        .iter()
        .any(|message| message.contains("has no shader target")));
}

#[test]
fn material_graph_compile_operation_reports_diagnostics_without_material() {
    let graph = graph_with_shader();

    let report = compile_material_graph_operation(&graph);

    assert!(report.material.is_none());
    assert!(report
        .diagnostics
        .iter()
        .any(|message| message.contains("has no output node")));
}

fn graph_with_shader() -> MaterialGraphAsset {
    MaterialGraphAsset {
        uri: AssetUri::parse("res://materials/test.material_graph.toml").unwrap(),
        name: "Test Material".to_string(),
        shader: Some(asset_ref("res://shaders/pbr.wgsl")),
        nodes: Vec::new(),
        links: Vec::new(),
        parameters: BTreeMap::new(),
    }
}

fn node(id: &str, kind: MaterialGraphNodeKindAsset) -> MaterialGraphNodeAsset {
    MaterialGraphNodeAsset {
        id: id.to_string(),
        position: [0.0, 0.0],
        kind,
    }
}

fn link(from_node: &str, from_pin: &str, to_node: &str, to_pin: &str) -> MaterialGraphLinkAsset {
    MaterialGraphLinkAsset {
        from_node: from_node.to_string(),
        from_pin: from_pin.to_string(),
        to_node: to_node.to_string(),
        to_pin: to_pin.to_string(),
    }
}

fn asset_ref(locator: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(locator).unwrap())
}
