use std::collections::BTreeSet;

use super::{
    MaterialGraphAsset, MaterialGraphDiagnostic, MaterialGraphLinkAsset, MaterialGraphNodeAsset,
    MaterialGraphNodeKindAsset,
};

/// A source-borrowed delete candidate, not mutation or undo authority.
///
/// Indices identify occurrences in this exact source revision; they are not
/// persistent IDs. Nodes and incident links retain source order, including
/// duplicate link occurrences, so the host can prepare one complete delta and
/// inverse in its existing primary history. Parameters remain untouched: another
/// declaration may use the same key, and an orphan override is still author data.
///
/// The host must retain the actual document/subdocument/source revision and
/// retention guard, then use one prepare/permit/CAS. Confirmation, selection and
/// focus, an expression preview reset, diagnostics and rendering belong to that
/// transaction's real host. This candidate creates no model, IDs or history.
#[derive(Debug)]
pub struct MaterialGraphNodeDeletionPlan<'a> {
    source: &'a MaterialGraphAsset,
    nodes: Vec<(usize, &'a MaterialGraphNodeAsset)>,
    links: Vec<(usize, &'a MaterialGraphLinkAsset)>,
    protected_nodes: Vec<&'a MaterialGraphNodeAsset>,
}

impl<'a> MaterialGraphNodeDeletionPlan<'a> {
    pub fn source(&self) -> &'a MaterialGraphAsset {
        self.source
    }

    pub fn can_delete(&self) -> bool {
        !self.nodes.is_empty()
    }

    pub fn removed_nodes(&self) -> &[(usize, &'a MaterialGraphNodeAsset)] {
        &self.nodes
    }

    pub fn removed_links(&self) -> &[(usize, &'a MaterialGraphLinkAsset)] {
        &self.links
    }

    pub fn protected_nodes(&self) -> &[&'a MaterialGraphNodeAsset] {
        &self.protected_nodes
    }
}

/// Plans deletion from exact stable node keys in the current source.
///
/// Like UE's CanDeleteNodes/DeleteNodesInternal, a mixed selection may delete
/// expressions while keeping the material Output root. Empty/root-only
/// selections are visible no-ops. Stale, duplicate or ambiguous identities reject
/// the whole candidate before any partial deletion. Compile errors, cycles and
/// malformed links do not prevent removing an expression to repair the graph.
pub fn plan_material_graph_node_deletion<'a>(
    graph: &'a MaterialGraphAsset,
    selected_node_ids: &[&str],
) -> Result<MaterialGraphNodeDeletionPlan<'a>, Vec<MaterialGraphDiagnostic>> {
    let mut selected = BTreeSet::new();
    for id in selected_node_ids {
        if id.trim().is_empty() || !selected.insert(*id) {
            return Err(vec![MaterialGraphDiagnostic::graph(
                "material graph deletion requires distinct non-empty selected node IDs",
            )]);
        }
    }

    let mut plan = MaterialGraphNodeDeletionPlan {
        source: graph,
        nodes: Vec::new(),
        links: Vec::new(),
        protected_nodes: Vec::new(),
    };
    if selected.is_empty() {
        return Ok(plan);
    }
    let mut known = BTreeSet::new();
    let mut removed = BTreeSet::new();
    for (index, node) in graph.nodes.iter().enumerate() {
        if node.id.trim().is_empty() || !known.insert(node.id.as_str()) {
            return Err(vec![MaterialGraphDiagnostic::graph(
                "material graph deletion requires unambiguous non-empty source node IDs",
            )]);
        }
        if !selected.contains(node.id.as_str()) {
            continue;
        }
        match &node.kind {
            MaterialGraphNodeKindAsset::Output => plan.protected_nodes.push(node),
            MaterialGraphNodeKindAsset::ScalarParameter { .. }
            | MaterialGraphNodeKindAsset::VectorParameter { .. }
            | MaterialGraphNodeKindAsset::TextureSample { .. }
            | MaterialGraphNodeKindAsset::Add
            | MaterialGraphNodeKindAsset::Multiply => {
                removed.insert(node.id.as_str());
                plan.nodes.push((index, node));
            }
        }
    }
    if !selected.is_subset(&known) {
        return Err(vec![MaterialGraphDiagnostic::graph(
            "material graph deletion selection contains a node that is no longer present",
        )]);
    }
    for (index, link) in graph.links.iter().enumerate() {
        if removed.contains(link.from_node.as_str()) || removed.contains(link.to_node.as_str()) {
            plan.links.push((index, link));
        }
    }
    Ok(plan)
}
