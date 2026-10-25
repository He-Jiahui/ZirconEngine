use std::collections::{BTreeMap, VecDeque};

use super::{
    MaterialGraphAsset, MaterialGraphDiagnostic, MaterialGraphIndex, MaterialGraphNodeKindAsset,
    MaterialGraphParameterAsset,
};

pub(super) const OUTPUT_PINS: &[(&str, &str)] = &[
    ("base_color", "vec4"),
    ("metallic", "float"),
    ("roughness", "float"),
    ("normal", "vec4"),
    ("emissive", "vec4"),
];

pub(super) fn output_pin(kind: &MaterialGraphNodeKindAsset) -> Option<&'static str> {
    match kind {
        MaterialGraphNodeKindAsset::Output => None,
        MaterialGraphNodeKindAsset::TextureSample { .. } => Some("color"),
        _ => Some("value"),
    }
}

pub(super) fn has_input_pin(kind: &MaterialGraphNodeKindAsset, pin: &str) -> bool {
    match kind {
        MaterialGraphNodeKindAsset::Output => OUTPUT_PINS.iter().any(|(name, _)| *name == pin),
        MaterialGraphNodeKindAsset::Add | MaterialGraphNodeKindAsset::Multiply => {
            matches!(pin, "a" | "b")
        }
        _ => false,
    }
}

pub(super) fn validate(
    graph: &MaterialGraphAsset,
    index: &MaterialGraphIndex<'_>,
    diagnostics: &mut Vec<MaterialGraphDiagnostic>,
) {
    for link in &graph.links {
        if let Some(node) = index.node(&link.from_node) {
            if output_pin(&node.kind) != Some(link.from_pin.as_str()) {
                diagnostics.push(MaterialGraphDiagnostic::node(
                    &link.from_node,
                    format!(
                        "material graph has no output pin `{}.{}`",
                        link.from_node, link.from_pin
                    ),
                ));
            }
        }
        if let Some(node) = index.node(&link.to_node) {
            if !has_input_pin(&node.kind, &link.to_pin) {
                diagnostics.push(MaterialGraphDiagnostic::node(
                    &link.to_node,
                    format!(
                        "material graph has no input pin `{}.{}`",
                        link.to_node, link.to_pin
                    ),
                ));
            }
        }
    }
    for node in &graph.nodes {
        let (name, finite, matching_override) =
            match &node.kind {
                MaterialGraphNodeKindAsset::ScalarParameter { name, default } => (
                    name,
                    default.is_finite(),
                    graph.parameters.get(name).is_none_or(|value| {
                        matches!(value, MaterialGraphParameterAsset::Scalar(_))
                    }),
                ),
                MaterialGraphNodeKindAsset::VectorParameter { name, default } => (
                    name,
                    default.iter().all(|value| value.is_finite()),
                    graph.parameters.get(name).is_none_or(|value| {
                        matches!(value, MaterialGraphParameterAsset::Vector(_))
                    }),
                ),
                _ => continue,
            };
        if name.trim().is_empty() {
            diagnostics.push(MaterialGraphDiagnostic::node(
                &node.id,
                format!(
                    "material graph node `{}` has an empty parameter name",
                    node.id
                ),
            ));
        }
        if !finite {
            diagnostics.push(MaterialGraphDiagnostic::node(
                &node.id,
                format!("material graph parameter `{name}` has a non-finite default"),
            ));
        }
        if !matching_override {
            diagnostics.push(MaterialGraphDiagnostic::parameter(
                name,
                format!("material graph parameter `{name}` has an incompatible override type"),
            ));
        }
    }
    for (name, value) in &graph.parameters {
        let finite = match value {
            MaterialGraphParameterAsset::Scalar(value) => value.is_finite(),
            MaterialGraphParameterAsset::Vector(value) => {
                value.iter().all(|value| value.is_finite())
            }
        };
        if !finite {
            diagnostics.push(MaterialGraphDiagnostic::parameter(
                name,
                format!("material graph parameter `{name}` has a non-finite override"),
            ));
        }
    }
    validate_acyclic(graph, index, diagnostics);
    if diagnostics.is_empty() {
        if let Err(error) = super::connections::validate_types(graph) {
            diagnostics.push(error);
        }
    }
}

fn validate_acyclic(
    graph: &MaterialGraphAsset,
    index: &MaterialGraphIndex<'_>,
    diagnostics: &mut Vec<MaterialGraphDiagnostic>,
) {
    // Count all components, including disconnected nodes. Keep graph validation
    // iterative so a malformed long chain cannot overflow the call stack.
    let mut degrees: BTreeMap<_, usize> = index.nodes.keys().map(|id| (*id, 0)).collect();
    let mut outgoing: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for link in &graph.links {
        if index.node(&link.from_node).is_some() {
            if let Some(degree) = degrees.get_mut(link.to_node.as_str()) {
                *degree += 1;
                outgoing
                    .entry(&link.from_node)
                    .or_default()
                    .push(&link.to_node);
            }
        }
    }
    let mut ready: VecDeque<_> = degrees
        .iter()
        .filter_map(|(id, degree)| (*degree == 0).then_some(*id))
        .collect();
    let mut visited = 0;
    while let Some(id) = ready.pop_front() {
        visited += 1;
        for target in outgoing.get(id).into_iter().flatten() {
            let degree = degrees.get_mut(target).expect("indexed link target");
            *degree -= 1;
            if *degree == 0 {
                ready.push_back(*target);
            }
        }
    }
    if visited != degrees.len() {
        diagnostics.push(MaterialGraphDiagnostic::graph(
            "material graph contains a cycle",
        ));
    }
}
