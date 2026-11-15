use std::collections::BTreeSet;

use super::{graph_contract, MaterialGraphAsset, MaterialGraphIndex, MaterialGraphNodeKindAsset};

/// A location in the exact material graph supplied to validation or compilation.
///
/// Node IDs and pin/parameter keys are source schema keys, not display labels.
/// The host must bind these to its document/request/revision and reject stale
/// results before resolving a SourceLocator or changing selection/focus. This
/// value is not a retained document identity or a prepared command permit.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MaterialGraphDiagnosticTarget {
    Graph,
    Node {
        node_id: String,
    },
    Input {
        node_id: String,
        pin_name: String,
    },
    Output {
        node_id: String,
        pin_name: String,
    },
    /// A key in the input graph's parameter map; renames need host source remapping.
    Parameter {
        name: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaterialGraphDiagnostic {
    pub message: String,
    pub target: MaterialGraphDiagnosticTarget,
}

impl MaterialGraphDiagnostic {
    pub(super) fn graph(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            target: MaterialGraphDiagnosticTarget::Graph,
        }
    }

    pub(super) fn node(node_id: &str, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            target: MaterialGraphDiagnosticTarget::Node {
                node_id: node_id.into(),
            },
        }
    }

    pub(super) fn input(node_id: &str, pin_name: &str, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            target: MaterialGraphDiagnosticTarget::Input {
                node_id: node_id.into(),
                pin_name: pin_name.into(),
            },
        }
    }

    pub(super) fn output(node_id: &str, pin_name: &str, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            target: MaterialGraphDiagnosticTarget::Output {
                node_id: node_id.into(),
                pin_name: pin_name.into(),
            },
        }
    }

    pub(super) fn parameter(name: &str, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            target: MaterialGraphDiagnosticTarget::Parameter { name: name.into() },
        }
    }
}

pub fn validate_material_graph(graph: &MaterialGraphAsset) -> Vec<String> {
    messages(validate_material_graph_with_diagnostics(graph))
}

/// Validates without modifying the input; equal messages at distinct source
/// locations remain separate. The legacy String API still deduplicates messages.
pub fn validate_material_graph_with_diagnostics(
    graph: &MaterialGraphAsset,
) -> Vec<MaterialGraphDiagnostic> {
    validate_with_index(graph, &MaterialGraphIndex::new(graph))
}

pub(super) fn messages(diagnostics: Vec<MaterialGraphDiagnostic>) -> Vec<String> {
    let mut messages: Vec<_> = diagnostics.into_iter().map(|entry| entry.message).collect();
    // Validation is already sorted. Do not reorder the compile-only shader
    // diagnostic appended after validation, or the scalar output error context.
    messages.dedup();
    messages
}

pub(super) fn validate_with_index(
    graph: &MaterialGraphAsset,
    index: &MaterialGraphIndex<'_>,
) -> Vec<MaterialGraphDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen_ids = BTreeSet::new();
    let mut node_ids = BTreeSet::new();
    let mut output_nodes = Vec::new();

    for (node_index, node) in graph.nodes.iter().enumerate() {
        if node.id.trim().is_empty() {
            diagnostics.push(MaterialGraphDiagnostic::node(
                &node.id,
                format!("material graph node at index {node_index} has an empty id"),
            ));
            continue;
        }
        if !seen_ids.insert(node.id.as_str()) {
            diagnostics.push(MaterialGraphDiagnostic::graph(format!(
                "material graph has duplicate node `{}`",
                node.id
            )));
        }
        node_ids.insert(node.id.as_str());
        if matches!(&node.kind, MaterialGraphNodeKindAsset::Output) {
            output_nodes.push(node.id.as_str());
        }
    }

    match output_nodes.len() {
        0 => diagnostics.push(MaterialGraphDiagnostic::graph(format!(
            "material graph `{}` has no output node",
            graph.name
        ))),
        1 => {
            if index.incoming_link(output_nodes[0], "base_color").is_none() {
                diagnostics.push(MaterialGraphDiagnostic::input(
                    output_nodes[0],
                    "base_color",
                    format!(
                        "material graph output `{}` is missing required input `base_color`",
                        output_nodes[0]
                    ),
                ));
            }
        }
        _ => diagnostics.push(MaterialGraphDiagnostic::graph(
            "material graph must contain exactly one output node",
        )),
    }

    for (link_index, link) in graph.links.iter().enumerate() {
        if !node_ids.contains(link.from_node.as_str()) {
            diagnostics.push(MaterialGraphDiagnostic::graph(format!(
                "material graph link references missing source node `{}`",
                link.from_node
            )));
        }
        if !node_ids.contains(link.to_node.as_str()) {
            diagnostics.push(MaterialGraphDiagnostic::graph(format!(
                "material graph link references missing target node `{}`",
                link.to_node
            )));
        }
        if link.from_pin.trim().is_empty() {
            diagnostics.push(MaterialGraphDiagnostic::output(
                &link.from_node,
                &link.from_pin,
                format!(
                    "material graph link {link_index} from `{}` has an empty output pin",
                    link.from_node
                ),
            ));
        }
        if link.to_pin.trim().is_empty() {
            diagnostics.push(MaterialGraphDiagnostic::input(
                &link.to_node,
                &link.to_pin,
                format!(
                    "material graph link {link_index} to `{}` has an empty input pin",
                    link.to_node
                ),
            ));
        }
    }

    for (node_id, pins) in &index.incoming_links {
        for (pin, entry) in pins {
            if entry.count > 1 {
                let message = format!("material graph has multiple links target `{node_id}.{pin}`");
                let diagnostic = match index.node(node_id) {
                    Some(node) if graph_contract::has_input_pin(&node.kind, pin) => {
                        MaterialGraphDiagnostic::input(node_id, pin, message)
                    }
                    Some(_) => MaterialGraphDiagnostic::node(node_id, message),
                    None => MaterialGraphDiagnostic::graph(message),
                };
                diagnostics.push(diagnostic);
            }
        }
    }

    for node in &graph.nodes {
        if matches!(
            &node.kind,
            MaterialGraphNodeKindAsset::Add | MaterialGraphNodeKindAsset::Multiply
        ) {
            for pin in ["a", "b"] {
                if index.incoming_link(&node.id, pin).is_none() {
                    diagnostics.push(MaterialGraphDiagnostic::input(
                        &node.id,
                        pin,
                        format!(
                            "material graph node `{}` is missing required input `{}`",
                            node.id, pin
                        ),
                    ));
                }
            }
        }
        if let MaterialGraphNodeKindAsset::Output = &node.kind {
            if let Some(link) = index.incoming_link(&node.id, "base_color") {
                if index.node(&link.from_node).is_none() {
                    diagnostics.push(MaterialGraphDiagnostic::input(
                        &node.id,
                        "base_color",
                        format!(
                            "material graph output `{}` base_color input is disconnected",
                            node.id
                        ),
                    ));
                }
            }
        }
    }

    graph_contract::validate(graph, index, &mut diagnostics);
    diagnostics.sort_by(|a, b| {
        a.message
            .cmp(&b.message)
            .then_with(|| a.target.cmp(&b.target))
    });
    diagnostics.dedup();
    diagnostics
}
