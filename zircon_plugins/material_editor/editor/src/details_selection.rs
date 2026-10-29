use std::collections::BTreeSet;

use super::{
    MaterialGraphAsset, MaterialGraphDiagnostic, MaterialGraphNodeAsset, MaterialGraphNodeKindAsset,
};

/// An object to inspect in this exact authoring source, not a compiled material.
///
/// UE inspects the material for an empty selection or the Output root, and the
/// original expression for a regular node. `Graph` retains the actual authored
/// graph and its shader/override presence. The real Toolkit must resolve that
/// source through its current document; this does not invent MaterialAsset or
/// Material Function objects, property schemas or a flattened effective value.
#[derive(Debug)]
pub enum MaterialGraphDetailsTarget<'a> {
    Graph(&'a MaterialGraphAsset),
    Expression(&'a MaterialGraphNodeAsset),
}

/// Source-borrowed Details objects and the original selected nodes.
///
/// Build on selection change, not for every paint. Source order is stable even
/// when the input selection has a different order. A graph Details target is
/// included once, while every selected root keeps its own original identity.
///
/// This is neither a selection store nor mutation/focus authority. Before
/// publishing to its existing Details view, the host must revalidate the same
/// view incarnation, document/subdocument, source revision and selection request.
/// SetObjects, focus, diagnostics navigation and editing/undo stay with that
/// host's existing retained input and prepare/permit/CAS/history lifecycle.
#[derive(Debug)]
pub struct MaterialGraphDetailsSelection<'a> {
    source: &'a MaterialGraphAsset,
    selected_nodes: Vec<&'a MaterialGraphNodeAsset>,
    targets: Vec<MaterialGraphDetailsTarget<'a>>,
}

impl<'a> MaterialGraphDetailsSelection<'a> {
    pub fn source(&self) -> &'a MaterialGraphAsset {
        self.source
    }

    pub fn selected_nodes(&self) -> &[&'a MaterialGraphNodeAsset] {
        &self.selected_nodes
    }

    pub fn targets(&self) -> &[MaterialGraphDetailsTarget<'a>] {
        &self.targets
    }

    pub fn has_expression(&self) -> bool {
        self.targets
            .iter()
            .any(|target| matches!(target, MaterialGraphDetailsTarget::Expression(_)))
    }
}

/// Resolves exact current node keys without compiling or changing the source.
///
/// Stale, repeated or ambiguous identities reject the whole projection: do not
/// publish a partial selection or fall back to a different document. Empty
/// selection can still inspect the graph when its node identities are invalid,
/// and incomplete/cyclic/ill-typed expressions remain inspectable for repair.
pub fn project_material_graph_details_selection<'a>(
    graph: &'a MaterialGraphAsset,
    selected_node_ids: &[&str],
) -> Result<MaterialGraphDetailsSelection<'a>, Vec<MaterialGraphDiagnostic>> {
    if selected_node_ids.is_empty() {
        return Ok(MaterialGraphDetailsSelection {
            source: graph,
            selected_nodes: Vec::new(),
            targets: vec![MaterialGraphDetailsTarget::Graph(graph)],
        });
    }

    let mut selected = BTreeSet::new();
    for id in selected_node_ids {
        if id.trim().is_empty() || !selected.insert(*id) {
            return Err(vec![MaterialGraphDiagnostic::graph(
                "material graph Details requires distinct non-empty selected node IDs",
            )]);
        }
    }

    let mut known = BTreeSet::new();
    let mut selection = MaterialGraphDetailsSelection {
        source: graph,
        selected_nodes: Vec::new(),
        targets: Vec::new(),
    };
    let mut includes_graph = false;
    for node in &graph.nodes {
        if node.id.trim().is_empty() || !known.insert(node.id.as_str()) {
            return Err(vec![MaterialGraphDiagnostic::graph(
                "material graph Details requires unambiguous non-empty source node IDs",
            )]);
        }
        if !selected.contains(node.id.as_str()) {
            continue;
        }
        selection.selected_nodes.push(node);
        match &node.kind {
            MaterialGraphNodeKindAsset::Output => {
                if !includes_graph {
                    selection
                        .targets
                        .push(MaterialGraphDetailsTarget::Graph(graph));
                    includes_graph = true;
                }
            }
            MaterialGraphNodeKindAsset::ScalarParameter { .. }
            | MaterialGraphNodeKindAsset::VectorParameter { .. }
            | MaterialGraphNodeKindAsset::TextureSample { .. }
            | MaterialGraphNodeKindAsset::Add
            | MaterialGraphNodeKindAsset::Multiply => selection
                .targets
                .push(MaterialGraphDetailsTarget::Expression(node)),
        }
    }
    if !selected.is_subset(&known) {
        return Err(vec![MaterialGraphDiagnostic::graph(
            "material graph Details selection contains a node that is no longer present",
        )]);
    }
    Ok(selection)
}
