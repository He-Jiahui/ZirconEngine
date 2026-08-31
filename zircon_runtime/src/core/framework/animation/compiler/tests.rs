use crate::core::framework::animation::{
    AnimationGraphAsset, AnimationGraphNodeAsset, AnimationGraphParameterAsset,
    AnimationParameterValue,
};
use crate::core::resource::{AssetReference, ResourceLocator};

use super::{
    compile_animation_graph, compile_animation_source, AnimationCompileElement,
    AnimationCompileProduct, AnimationCompileSeverity, AnimationCompileSource,
    AnimationCompiledGraphNode, AnimationCompilerAssetKind, AnimationCompilerSchemaRegistry,
    AnimationGraphNodeSchemaKind, AnimationGraphPinDirection,
};

fn graph_asset(nodes: Vec<AnimationGraphNodeAsset>) -> AnimationGraphAsset {
    AnimationGraphAsset {
        name: Some("Locomotion".to_string()),
        parameters: vec![AnimationGraphParameterAsset {
            name: "weight".to_string(),
            default_value: AnimationParameterValue::Scalar(0.5),
        }],
        nodes,
    }
}

fn clip_node(id: &str) -> AnimationGraphNodeAsset {
    AnimationGraphNodeAsset::Clip {
        id: id.to_string(),
        clip: AssetReference::from_locator(
            ResourceLocator::parse("res://animation/idle.clip").unwrap(),
        ),
        playback_speed: 1.0,
        looping: true,
    }
}

#[test]
fn graph_compiler_resolves_references_to_a_dependency_first_index_ir() {
    let graph = graph_asset(vec![
        clip_node("idle"),
        AnimationGraphNodeAsset::Blend {
            id: "locomotion".to_string(),
            inputs: vec!["idle".to_string()],
            weight_parameter: Some("weight".to_string()),
        },
        AnimationGraphNodeAsset::Output {
            source: "locomotion".to_string(),
        },
    ]);

    let compilation = compile_animation_graph(&graph);

    assert!(compilation.diagnostics().is_empty());
    let artifact = compilation
        .artifact()
        .expect("valid graph must yield a compiled artifact");
    assert_eq!(
        artifact.node_ids(),
        &["idle".to_string(), "locomotion".to_string()]
    );
    assert_eq!(artifact.output_node(), 1);
    assert_eq!(artifact.evaluation_order(), &[0, 1]);
    let AnimationCompiledGraphNode::Blend {
        weight_parameter, ..
    } = &artifact.nodes()[1]
    else {
        panic!("second graph node must be the compiled blend");
    };
    assert_eq!(*weight_parameter, Some(0));
}

#[test]
fn graph_compiler_rejects_duplicate_ids_missing_references_and_cycles() {
    let graph = graph_asset(vec![
        AnimationGraphNodeAsset::Blend {
            id: "cycle".to_string(),
            inputs: vec!["missing".to_string(), "cycle".to_string()],
            weight_parameter: None,
        },
        clip_node("cycle"),
        AnimationGraphNodeAsset::Output {
            source: "cycle".to_string(),
        },
    ]);

    let compilation = compile_animation_graph(&graph);

    assert!(compilation.artifact().is_none());
    assert!(compilation.diagnostics().iter().any(|diagnostic| {
        diagnostic.code() == "ZR-ANIM-COMP-GRAPH-002"
            && diagnostic.element() == &AnimationCompileElement::GraphNode("cycle".to_string())
            && diagnostic.severity() == AnimationCompileSeverity::Error
    }));
    assert!(compilation
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code() == "ZR-ANIM-COMP-GRAPH-006"));
    assert!(compilation
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code() == "ZR-ANIM-COMP-GRAPH-009"));
}

#[test]
fn graph_compiler_keeps_unreachable_nodes_as_diagnostics_without_rejecting_the_artifact() {
    let graph = graph_asset(vec![
        clip_node("idle"),
        clip_node("unused"),
        AnimationGraphNodeAsset::Output {
            source: "idle".to_string(),
        },
    ]);

    let compilation = compile_animation_graph(&graph);

    assert!(compilation.artifact().is_some());
    assert!(compilation.diagnostics().iter().any(|diagnostic| {
        diagnostic.code() == "ZR-ANIM-COMP-GRAPH-010"
            && diagnostic.element() == &AnimationCompileElement::GraphNode("unused".to_string())
            && diagnostic.severity() == AnimationCompileSeverity::Warning
    }));
}

#[test]
fn graph_compiler_handles_deep_graphs_without_recursive_traversal() {
    const DEEP_GRAPH_NODES: usize = 4096;

    let mut nodes = Vec::with_capacity(DEEP_GRAPH_NODES + 1);
    nodes.push(clip_node("node_0"));
    for index in 1..DEEP_GRAPH_NODES {
        nodes.push(AnimationGraphNodeAsset::Blend {
            id: format!("node_{index}"),
            inputs: vec![format!("node_{}", index - 1)],
            weight_parameter: None,
        });
    }
    nodes.push(AnimationGraphNodeAsset::Output {
        source: format!("node_{}", DEEP_GRAPH_NODES - 1),
    });

    let compilation = compile_animation_graph(&graph_asset(nodes));

    let artifact = compilation
        .artifact()
        .expect("a deep acyclic graph must compile");
    assert_eq!(artifact.evaluation_order().len(), DEEP_GRAPH_NODES);
    assert_eq!(artifact.evaluation_order().first(), Some(&0));
    assert_eq!(
        artifact.evaluation_order().last(),
        Some(&(DEEP_GRAPH_NODES - 1))
    );
}

#[test]
fn unified_compiler_dispatch_preserves_the_graph_product_kind() {
    let graph = graph_asset(vec![
        clip_node("idle"),
        AnimationGraphNodeAsset::Output {
            source: "idle".to_string(),
        },
    ]);

    let product = compile_animation_source(AnimationCompileSource::Graph(&graph));

    assert!(matches!(
        product,
        AnimationCompileProduct::Graph(compilation) if compilation.artifact().is_some()
    ));
}

#[test]
fn builtin_schema_registry_owns_all_current_asset_kinds_and_graph_pin_contracts() {
    assert_eq!(
        AnimationCompilerSchemaRegistry::BUILTIN_OWNER.id(),
        "zircon.runtime.animation"
    );
    assert_eq!(
        AnimationCompilerSchemaRegistry::supported_asset_kinds(),
        &[
            AnimationCompilerAssetKind::Sequence,
            AnimationCompilerAssetKind::Graph,
            AnimationCompilerAssetKind::StateMachine,
        ]
    );
    let blend = AnimationCompilerSchemaRegistry::graph_node(AnimationGraphNodeSchemaKind::Blend);
    assert!(blend.inputs().iter().any(|pin| {
        pin.id() == "inputs"
            && pin.direction() == AnimationGraphPinDirection::Input
            && pin.accepts_multiple()
    }));
    let output = AnimationCompilerSchemaRegistry::graph_node(AnimationGraphNodeSchemaKind::Output);
    assert!(output.outputs().is_empty());
    assert!(output.inputs().iter().all(|pin| pin.required()));
}
