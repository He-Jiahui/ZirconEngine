use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;
use zircon_editor::core::editor_extension::EditorExtensionRegistry;
use zircon_editor::core::editor_operation::EditorOperationPath;
use zircon_editor::EditorPlugin;
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialGraphAsset, MaterialGraphLinkAsset, MaterialGraphNodeAsset,
    MaterialGraphNodeKindAsset, MaterialGraphParameterAsset,
};
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime::plugin::PluginModuleKind;

#[test]
fn material_authoring_registration_exposes_menu_items_and_payload_schemas() {
    let mut registry = EditorExtensionRegistry::default();
    editor_plugin()
        .register_editor_extensions(&mut registry)
        .expect("material authoring registration");
    let operation =
        EditorOperationPath::parse("material_editor.graph.compile").expect("valid material path");
    let descriptor = registry
        .commands()
        .command(&operation)
        .expect("compile operation registered");

    assert_eq!(
        descriptor
            .menu_path()
            .expect("compile command menu path")
            .stable_path(),
        "plugins/material_editor/material_editor.graph.compile"
    );
    assert_eq!(
        descriptor.payload_schema_id(),
        Some("material_editor.compile_graph.v1")
    );
    assert!(registry.menu_items().is_empty());
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
        vec![zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost]
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
fn material_graph_validation_rejects_multiple_links_to_one_input_pin() {
    let mut graph = graph_with_shader();
    graph.nodes.push(node(
        "first",
        MaterialGraphNodeKindAsset::VectorParameter {
            name: "first".to_string(),
            default: [0.25, 0.5, 0.75, 1.0],
        },
    ));
    graph.nodes.push(node(
        "second",
        MaterialGraphNodeKindAsset::VectorParameter {
            name: "second".to_string(),
            default: [1.0, 0.0, 0.0, 1.0],
        },
    ));
    graph
        .nodes
        .push(node("output", MaterialGraphNodeKindAsset::Output));
    graph
        .links
        .push(link("first", "value", "output", "base_color"));
    graph
        .links
        .push(link("second", "value", "output", "base_color"));

    let diagnostics = validate_material_graph(&graph);

    assert!(diagnostics
        .iter()
        .any(|message| message.contains("multiple links target `output.base_color`")));
    assert!(compile_material_graph(&graph).is_err());
}

#[test]
#[ignore = "release performance gate"]
fn material_graph_indexed_evaluation_release_gate_avoids_recursive_map_rebuilds() {
    const SAMPLE_PAIRS: usize = 21;
    const MATH_NODES: usize = 256;
    const EVALUATIONS_PER_SAMPLE: usize = 2;
    const LEGACY_NODE_MAP_BUILDS_PER_EVALUATION: usize = 513;
    const REQUIRED_IMPROVEMENT_PERCENT: u128 = 80;

    let graph = indexed_math_chain(MATH_NODES);
    let expected = optimized_evaluate_material_graph(&graph);
    assert_eq!(
        expected,
        MaterialColorInput::Constant([0.25, 0.5, 0.75, 1.0])
    );
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure_material_evaluation(
                &graph,
                EVALUATIONS_PER_SAMPLE,
                legacy_evaluate_material_graph,
            ));
            optimized_samples.push(measure_material_evaluation(
                &graph,
                EVALUATIONS_PER_SAMPLE,
                optimized_evaluate_material_graph,
            ));
        } else {
            optimized_samples.push(measure_material_evaluation(
                &graph,
                EVALUATIONS_PER_SAMPLE,
                optimized_evaluate_material_graph,
            ));
            legacy_samples.push(measure_material_evaluation(
                &graph,
                EVALUATIONS_PER_SAMPLE,
                legacy_evaluate_material_graph,
            ));
        }
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples).as_nanos();
    let optimized_p95 = nearest_rank_p95(&optimized_samples).as_nanos();
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT plugins08_material_indexed_graph sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even math_nodes={MATH_NODES} evaluations_per_sample={EVALUATIONS_PER_SAMPLE} legacy_node_map_builds_per_evaluation={LEGACY_NODE_MAP_BUILDS_PER_EVALUATION} optimized_node_map_builds_per_evaluation=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={REQUIRED_IMPROVEMENT_PERCENT}",
        durations_csv(&legacy_samples),
        durations_csv(&optimized_samples)
    );
    assert!(
        improvement_percent >= REQUIRED_IMPROVEMENT_PERCENT,
        "indexed graph evaluation must improve P95 by at least {REQUIRED_IMPROVEMENT_PERCENT}% (legacy={legacy_p95}ns optimized={optimized_p95}ns improvement={improvement_percent}%)"
    );
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

fn indexed_math_chain(math_nodes: usize) -> MaterialGraphAsset {
    let mut graph = graph_with_shader();
    graph.nodes.push(node(
        "seed",
        MaterialGraphNodeKindAsset::VectorParameter {
            name: "seed".to_string(),
            default: [0.25, 0.5, 0.75, 1.0],
        },
    ));
    graph.nodes.push(node(
        "one",
        MaterialGraphNodeKindAsset::ScalarParameter {
            name: "one".to_string(),
            default: 1.0,
        },
    ));
    let mut previous = "seed".to_string();
    for index in 0..math_nodes {
        let current = format!("multiply_{index:04}");
        graph
            .nodes
            .push(node(&current, MaterialGraphNodeKindAsset::Multiply));
        graph.links.push(link(&previous, "value", &current, "a"));
        graph.links.push(link("one", "value", &current, "b"));
        previous = current;
    }
    graph
        .nodes
        .push(node("output", MaterialGraphNodeKindAsset::Output));
    graph
        .links
        .push(link(&previous, "value", "output", "base_color"));
    graph
}

fn optimized_evaluate_material_graph(graph: &MaterialGraphAsset) -> MaterialColorInput {
    let index = MaterialGraphIndex::new(graph);
    let output = graph
        .nodes
        .iter()
        .find(|node| matches!(&node.kind, MaterialGraphNodeKindAsset::Output))
        .unwrap();
    let base_color = index.incoming_link(&output.id, "base_color").unwrap();
    evaluate_color_input(graph, &index, &base_color.from_node, &mut BTreeSet::new()).unwrap()
}

fn legacy_evaluate_material_graph(graph: &MaterialGraphAsset) -> MaterialColorInput {
    let output = graph
        .nodes
        .iter()
        .find(|node| matches!(&node.kind, MaterialGraphNodeKindAsset::Output))
        .unwrap();
    let base_color = graph
        .links
        .iter()
        .find(|link| link.to_node == output.id && link.to_pin == "base_color")
        .unwrap();
    legacy_evaluate_color_input(graph, &base_color.from_node, &mut BTreeSet::new()).unwrap()
}

fn legacy_evaluate_color_input(
    graph: &MaterialGraphAsset,
    node_id: &str,
    evaluating: &mut BTreeSet<String>,
) -> Result<MaterialColorInput, String> {
    if !evaluating.insert(node_id.to_string()) {
        return Err("cycle".to_string());
    }
    let nodes = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let result = match nodes.get(node_id).map(|node| &node.kind) {
        Some(MaterialGraphNodeKindAsset::ScalarParameter { name, default }) => {
            let value = match graph.parameters.get(name) {
                Some(MaterialGraphParameterAsset::Scalar(value)) => *value,
                _ => *default,
            };
            Ok(MaterialColorInput::Constant([value, value, value, 1.0]))
        }
        Some(MaterialGraphNodeKindAsset::VectorParameter { name, default }) => {
            let value = match graph.parameters.get(name) {
                Some(MaterialGraphParameterAsset::Vector(value)) => *value,
                _ => *default,
            };
            Ok(MaterialColorInput::Constant(value))
        }
        Some(MaterialGraphNodeKindAsset::Multiply) => {
            let a = legacy_evaluate_color_pin(graph, node_id, "a", evaluating)?;
            let b = legacy_evaluate_color_pin(graph, node_id, "b", evaluating)?;
            combine_color_inputs("multiply", a, b, |left, right| left * right)
        }
        _ => Err(format!("unsupported benchmark node `{node_id}`")),
    };
    evaluating.remove(node_id);
    result
}

fn legacy_evaluate_color_pin(
    graph: &MaterialGraphAsset,
    node_id: &str,
    pin: &str,
    evaluating: &mut BTreeSet<String>,
) -> Result<MaterialColorInput, String> {
    let link = graph
        .links
        .iter()
        .find(|link| link.to_node == node_id && link.to_pin == pin)
        .unwrap();
    legacy_evaluate_color_input(graph, &link.from_node, evaluating)
}

fn measure_material_evaluation(
    graph: &MaterialGraphAsset,
    evaluations_per_sample: usize,
    evaluate: fn(&MaterialGraphAsset) -> MaterialColorInput,
) -> Duration {
    let started = Instant::now();
    for _ in 0..evaluations_per_sample {
        assert_eq!(
            black_box(evaluate(black_box(graph))),
            MaterialColorInput::Constant([0.25, 0.5, 0.75, 1.0])
        );
    }
    started.elapsed()
}

fn nearest_rank_p95(samples: &[Duration]) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[((sorted.len() * 95).div_ceil(100)).saturating_sub(1)]
}

fn durations_csv(samples: &[Duration]) -> String {
    samples
        .iter()
        .map(|sample| sample.as_nanos().to_string())
        .collect::<Vec<_>>()
        .join(",")
}
