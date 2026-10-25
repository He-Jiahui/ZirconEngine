use std::collections::{BTreeMap, VecDeque};

use super::{
    graph_contract, MaterialGraphAsset, MaterialGraphDiagnostic, MaterialGraphLinkAsset,
    MaterialGraphNodeAsset, MaterialGraphNodeKindAsset,
};

/// Value categories implemented by the current material compiler, not shader pin labels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaterialGraphValueType {
    Scalar,
    Vector,
    Texture,
    /// An incomplete math node needs another input before its category is known.
    Undetermined,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MaterialGraphConnectionAction<'a> {
    Connect,
    /// The original edge to remove in the same domain transaction as the new edge.
    Replace(&'a MaterialGraphLinkAsset),
    Unchanged,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialGraphConnectionPlan<'a> {
    pub action: MaterialGraphConnectionAction<'a>,
    pub source_type: MaterialGraphValueType,
    pub has_undetermined_types: bool,
}

/// Checks the prospective graph without changing the authoring document.
///
/// An occupied input is replaced before checking cycles and downstream types.
/// Missing inputs are allowed while editing; this is not a compile-readiness check.
/// The host must bind the plan to its retained document revision and apply one
/// prepared domain delta with its inverse. This plan is not a revision permit.
pub fn can_connect_material_graph<'a>(
    graph: &'a MaterialGraphAsset,
    candidate: &MaterialGraphLinkAsset,
) -> Result<MaterialGraphConnectionPlan<'a>, String> {
    connection_plan(graph, candidate, None)
}

pub(super) fn can_connect_material_graph_with_node<'a>(
    graph: &'a MaterialGraphAsset,
    candidate: &MaterialGraphLinkAsset,
    node: &MaterialGraphNodeAsset,
) -> Result<MaterialGraphConnectionPlan<'a>, String> {
    connection_plan(graph, candidate, Some(node))
}

fn connection_plan<'a>(
    graph: &'a MaterialGraphAsset,
    candidate: &MaterialGraphLinkAsset,
    additional_node: Option<&MaterialGraphNodeAsset>,
) -> Result<MaterialGraphConnectionPlan<'a>, String> {
    let mut occupied = graph
        .links
        .iter()
        .filter(|link| link.to_node == candidate.to_node && link.to_pin == candidate.to_pin);
    let previous = occupied.next();
    if occupied.next().is_some() {
        return Err(format!(
            "material graph has multiple links target `{}.{}`",
            candidate.to_node, candidate.to_pin
        ));
    }
    let action = match previous {
        Some(link) if link == candidate => MaterialGraphConnectionAction::Unchanged,
        Some(link) => MaterialGraphConnectionAction::Replace(link),
        None => MaterialGraphConnectionAction::Connect,
    };
    let prospective = graph
        .links
        .iter()
        .filter(|link| link.to_node != candidate.to_node || link.to_pin != candidate.to_pin)
        .chain(std::iter::once(candidate));
    let nodes = graph.nodes.iter().chain(additional_node);
    let summary = analyze_types(nodes, prospective, Some(&candidate.from_node))
        .map_err(|error| error.message)?;
    Ok(MaterialGraphConnectionPlan {
        action,
        source_type: summary
            .source_type
            .ok_or_else(|| "material graph candidate has no output value".to_string())?,
        has_undetermined_types: summary.has_undetermined_types,
    })
}

pub(super) fn validate_types(graph: &MaterialGraphAsset) -> Result<(), MaterialGraphDiagnostic> {
    analyze_types(graph.nodes.iter(), graph.links.iter(), None).map(|_| ())
}

pub(super) fn resolved_node_types(
    graph: &MaterialGraphAsset,
) -> Result<BTreeMap<&str, MaterialGraphValueType>, MaterialGraphDiagnostic> {
    analyze_types(graph.nodes.iter(), graph.links.iter(), None).map(|summary| summary.types)
}

pub(super) fn validate_insertion(
    graph: &MaterialGraphAsset,
    node: &MaterialGraphNodeAsset,
    removed: &MaterialGraphLinkAsset,
    links: &[&MaterialGraphLinkAsset],
) -> Result<bool, String> {
    // The caller supplies the exact borrowed old edge, not an endpoint-wide
    // removal. Check the replacement or both insertion edges together; the
    // latter must expose the preserved source type to downstream constraints.
    let retained = graph
        .links
        .iter()
        .filter(|link| !std::ptr::eq(*link, removed));
    analyze_types(
        graph.nodes.iter().chain(std::iter::once(node)),
        retained.chain(links.iter().copied()),
        None,
    )
    .map(|summary| summary.has_undetermined_types)
    .map_err(|error| error.message)
}

struct TypeSummary<'a> {
    source_type: Option<MaterialGraphValueType>,
    has_undetermined_types: bool,
    types: BTreeMap<&'a str, MaterialGraphValueType>,
}

fn analyze_types<'a>(
    node_values: impl Iterator<Item = &'a MaterialGraphNodeAsset>,
    links: impl Iterator<Item = &'a MaterialGraphLinkAsset>,
    requested_source: Option<&str>,
) -> Result<TypeSummary<'a>, MaterialGraphDiagnostic> {
    // Borrow edges and nodes. These transient indices are not an authoring snapshot
    // or a history entry, and cover disconnected components as well as the output.
    let mut nodes = BTreeMap::new();
    for node in node_values {
        if node.id.trim().is_empty() {
            return Err(MaterialGraphDiagnostic::graph(
                "material graph node id must not be empty",
            ));
        }
        if nodes.insert(node.id.as_str(), node).is_some() {
            return Err(MaterialGraphDiagnostic::graph(format!(
                "material graph has duplicate node `{}`",
                node.id
            )));
        }
    }
    let mut incoming = BTreeMap::new();
    let mut outgoing: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut degrees: BTreeMap<_, usize> = nodes.keys().map(|id| (*id, 0)).collect();
    for link in links {
        let source = nodes.get(link.from_node.as_str()).ok_or_else(|| {
            MaterialGraphDiagnostic::graph(format!(
                "material graph link references missing source node `{}`",
                link.from_node
            ))
        })?;
        let target = nodes.get(link.to_node.as_str()).ok_or_else(|| {
            MaterialGraphDiagnostic::graph(format!(
                "material graph link references missing target node `{}`",
                link.to_node
            ))
        })?;
        if graph_contract::output_pin(&source.kind) != Some(link.from_pin.as_str()) {
            return Err(MaterialGraphDiagnostic::node(
                &link.from_node,
                format!(
                    "material graph has no output pin `{}.{}`",
                    link.from_node, link.from_pin
                ),
            ));
        }
        if !graph_contract::has_input_pin(&target.kind, &link.to_pin) {
            return Err(MaterialGraphDiagnostic::node(
                &link.to_node,
                format!(
                    "material graph has no input pin `{}.{}`",
                    link.to_node, link.to_pin
                ),
            ));
        }
        if link.from_node == link.to_node {
            return Err(MaterialGraphDiagnostic::node(
                &link.from_node,
                "material graph contains a cycle (self connection)",
            ));
        }
        if incoming
            .insert((link.to_node.as_str(), link.to_pin.as_str()), link)
            .is_some()
        {
            return Err(MaterialGraphDiagnostic::input(
                &link.to_node,
                &link.to_pin,
                format!(
                    "material graph has multiple links target `{}.{}`",
                    link.to_node, link.to_pin
                ),
            ));
        }
        *degrees
            .get_mut(link.to_node.as_str())
            .expect("indexed target") += 1;
        outgoing
            .entry(&link.from_node)
            .or_default()
            .push(&link.to_node);
    }
    let mut ready: VecDeque<_> = degrees
        .iter()
        .filter_map(|(id, degree)| (*degree == 0).then_some(*id))
        .collect();
    let mut types = BTreeMap::new();
    let mut visited = 0;
    let mut has_undetermined_types = false;
    while let Some(id) = ready.pop_front() {
        visited += 1;
        let input_type = |pin| {
            incoming
                .get(&(id, pin))
                .and_then(|link| types.get(link.from_node.as_str()))
                .copied()
                .unwrap_or(MaterialGraphValueType::Undetermined)
        };
        let value_type = match &nodes[id].kind {
            MaterialGraphNodeKindAsset::ScalarParameter { .. } => MaterialGraphValueType::Scalar,
            MaterialGraphNodeKindAsset::VectorParameter { .. } => MaterialGraphValueType::Vector,
            MaterialGraphNodeKindAsset::TextureSample { .. } => MaterialGraphValueType::Texture,
            MaterialGraphNodeKindAsset::Add | MaterialGraphNodeKindAsset::Multiply => {
                let a = input_type("a");
                let b = input_type("b");
                if a == MaterialGraphValueType::Texture || b == MaterialGraphValueType::Texture {
                    let op = if matches!(&nodes[id].kind, MaterialGraphNodeKindAsset::Add) {
                        "add"
                    } else {
                        "multiply"
                    };
                    return Err(MaterialGraphDiagnostic::input(
                        id,
                        if a == MaterialGraphValueType::Texture {
                            "a"
                        } else {
                            "b"
                        },
                        format!(
                            "material graph `{op}` node cannot combine texture-backed inputs in v1"
                        ),
                    ));
                }
                if a == MaterialGraphValueType::Vector || b == MaterialGraphValueType::Vector {
                    MaterialGraphValueType::Vector
                } else if a == MaterialGraphValueType::Scalar && b == MaterialGraphValueType::Scalar
                {
                    MaterialGraphValueType::Scalar
                } else {
                    MaterialGraphValueType::Undetermined
                }
            }
            MaterialGraphNodeKindAsset::Output => {
                for pin in ["metallic", "roughness"] {
                    if let Some(link) = incoming.get(&(id, pin)) {
                        if matches!(
                            input_type(pin),
                            MaterialGraphValueType::Vector | MaterialGraphValueType::Texture
                        ) {
                            return Err(MaterialGraphDiagnostic::output(
                                &link.from_node,
                                &link.from_pin,
                                format!("node `{}` does not produce a scalar", link.from_node),
                            ));
                        }
                    }
                }
                if let Some(link) = incoming.get(&(id, "normal")) {
                    if !matches!(
                        &nodes[link.from_node.as_str()].kind,
                        MaterialGraphNodeKindAsset::TextureSample { .. }
                    ) {
                        return Err(MaterialGraphDiagnostic::input(
                            id,
                            "normal",
                            "material graph `normal` requires a texture sample in v1",
                        ));
                    }
                }
                // Endpoint validation forbids outgoing links from an Output node.
                continue;
            }
        };
        has_undetermined_types |= value_type == MaterialGraphValueType::Undetermined;
        types.insert(id, value_type);
        for target in outgoing.get(id).into_iter().flatten() {
            let degree = degrees.get_mut(target).expect("indexed link target");
            *degree -= 1;
            if *degree == 0 {
                ready.push_back(*target);
            }
        }
    }
    if visited != nodes.len() {
        return Err(MaterialGraphDiagnostic::graph(
            "material graph contains a cycle",
        ));
    }
    Ok(TypeSummary {
        source_type: requested_source.and_then(|id| types.get(id)).copied(),
        has_undetermined_types,
        types,
    })
}
