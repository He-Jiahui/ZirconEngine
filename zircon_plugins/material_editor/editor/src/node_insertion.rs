use zircon_editor::{
    core::editor_authoring_extension::GraphNodeDescriptor,
    ui::graph::{GraphPinDirection, GraphPortRef},
};
use zircon_runtime_interface::resource::ResourceKind;

use super::{
    connections, graph_contract, validate_material_graph_node_creation, AssetReference,
    MaterialGraphAsset, MaterialGraphDiagnostic, MaterialGraphLinkAsset, MaterialGraphNodeAsset,
};

/// A read-only insertion candidate, not a document revision or mutation permit.
///
/// The existing edge and candidate node remain borrowed. Two links are ordered
/// old-source -> new-input, then new-output -> original-input. A one-link plan is
/// UE's explicit replacement fallback; the previous source node stays intact.
/// The actual host must retain its document/source/revision and publish insertion, removal
/// of this exact old edge and the one or two new edges in one prepared transaction
/// with one inverse in the existing primary history. No graph is cloned here.
#[derive(Debug, PartialEq)]
pub struct MaterialGraphNodeInsertionPlan<'a> {
    node: &'a MaterialGraphNodeAsset,
    removed: &'a MaterialGraphLinkAsset,
    links: Vec<MaterialGraphLinkAsset>,
    has_undetermined_types: bool,
}

impl<'a> MaterialGraphNodeInsertionPlan<'a> {
    pub fn node(&self) -> &'a MaterialGraphNodeAsset {
        self.node
    }

    pub fn removed_link(&self) -> &'a MaterialGraphLinkAsset {
        self.removed
    }

    pub fn links(&self) -> &[MaterialGraphLinkAsset] {
        &self.links
    }

    pub fn reconnects_previous_source(&self) -> bool {
        self.links.len() == 2
    }

    pub fn has_undetermined_types(&self) -> bool {
        self.has_undetermined_types
    }
}

/// Plans node creation and reconnection at one real, occupied input port.
///
/// Like UE's InsertNewNode, select the first legal new output, then try receiving
/// inputs in their existing registered order. Check both links together before
/// reconnecting the previous source. If no receiving input is legal, explicitly
/// return the one-link replacement fallback instead. The original source node
/// and its other connections are never removed. Incomplete math remains editable;
/// malformed source ports, incompatible new outputs and cycles are rejected.
pub fn plan_material_graph_node_insertion<'a>(
    graph: &'a MaterialGraphAsset,
    action: &GraphNodeDescriptor,
    node: &'a MaterialGraphNodeAsset,
    anchor: &GraphPortRef<String>,
    resolve_kind: impl FnMut(&AssetReference) -> Result<ResourceKind, String>,
) -> Result<MaterialGraphNodeInsertionPlan<'a>, Vec<MaterialGraphDiagnostic>> {
    if anchor.direction != GraphPinDirection::Input {
        return Err(vec![MaterialGraphDiagnostic::graph(
            "material graph insertion requires an occupied input port",
        )]);
    }
    let mut targets = graph.nodes.iter().filter(|node| node.id == anchor.node_id);
    let Some(target) = targets.next() else {
        return Err(vec![MaterialGraphDiagnostic::graph(
            "material graph insertion target node is no longer present",
        )]);
    };
    if target.id.trim().is_empty() || targets.next().is_some() {
        return Err(vec![MaterialGraphDiagnostic::graph(
            "material graph insertion target node identity is ambiguous",
        )]);
    }
    if !graph_contract::has_input_pin(&target.kind, &anchor.pin_name) {
        return Err(vec![MaterialGraphDiagnostic::node(
            &target.id,
            "material graph insertion target input no longer exists",
        )]);
    }
    let input_error = |message| {
        vec![MaterialGraphDiagnostic::input(
            &target.id,
            &anchor.pin_name,
            message,
        )]
    };
    let mut occupied = graph
        .links
        .iter()
        .filter(|edge| edge.to_node == target.id && edge.to_pin == anchor.pin_name);
    let Some(removed) = occupied.next() else {
        return Err(input_error(
            "material graph insertion target input is not connected".to_owned(),
        ));
    };
    if occupied.next().is_some() {
        return Err(input_error(
            "material graph insertion target has multiple existing connections".to_owned(),
        ));
    }
    let mut previous_sources = graph
        .nodes
        .iter()
        .filter(|node| node.id == removed.from_node);
    let Some(previous_source) = previous_sources.next() else {
        return Err(input_error(
            "material graph insertion previous source node is no longer present".into(),
        ));
    };
    if previous_sources.next().is_some() {
        return Err(vec![MaterialGraphDiagnostic::graph(
            "material graph insertion previous source node identity is ambiguous",
        )]);
    }
    if graph_contract::output_pin(&previous_source.kind) != Some(removed.from_pin.as_str()) {
        return Err(input_error(
            "material graph insertion previous source output no longer exists".into(),
        ));
    }
    let diagnostics = validate_material_graph_node_creation(graph, action, node, resolve_kind);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let mut first_error = None;
    for output in action.outputs() {
        let outgoing = MaterialGraphLinkAsset {
            from_node: node.id.clone(),
            from_pin: output.name().into(),
            to_node: target.id.clone(),
            to_pin: anchor.pin_name.clone(),
        };
        let fallback_types =
            match connections::validate_insertion(graph, node, removed, &[&outgoing]) {
                Ok(has_undetermined_types) => has_undetermined_types,
                Err(error) => {
                    first_error.get_or_insert(error);
                    continue;
                }
            };
        for input in action.inputs() {
            let incoming = MaterialGraphLinkAsset {
                from_node: removed.from_node.clone(),
                from_pin: removed.from_pin.clone(),
                to_node: node.id.clone(),
                to_pin: input.name().into(),
            };
            if let Ok(has_undetermined_types) =
                connections::validate_insertion(graph, node, removed, &[&incoming, &outgoing])
            {
                return Ok(MaterialGraphNodeInsertionPlan {
                    node,
                    removed,
                    links: vec![incoming, outgoing],
                    has_undetermined_types,
                });
            }
        }
        return Ok(MaterialGraphNodeInsertionPlan {
            node,
            removed,
            links: vec![outgoing],
            has_undetermined_types: fallback_types,
        });
    }
    Err(input_error(format!(
        "material graph cannot insert `{}` at this input: {}",
        action.id(),
        first_error.unwrap_or_else(|| "new node has no compatible output".into())
    )))
}
