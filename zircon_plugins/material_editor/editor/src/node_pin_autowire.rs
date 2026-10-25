use zircon_editor::{
    core::editor_authoring_extension::GraphNodeDescriptor,
    ui::graph::{GraphPinDirection, GraphPortRef},
};
use zircon_runtime_interface::resource::ResourceKind;

use super::{
    connections::can_connect_material_graph_with_node, graph_contract,
    validate_material_graph_node_creation, AssetReference, MaterialGraphAsset,
    MaterialGraphDiagnostic, MaterialGraphLinkAsset, MaterialGraphNodeAsset,
};

/// Suggests the first legal opposite pin in the current registered action order.
///
/// The anchor is an actual port in the supplied source graph, not a display name
/// or screenshot coordinate. The compiler's connection analyzer decides types;
/// static palette labels alone cannot describe scalar/vector math or textures.
/// `Ok(None)` permits creation without an automatic wire, including a node with
/// no opposite pins. It does not certify that the graph is ready to compile.
///
/// Dragging from an occupied input requires UE's multi-edge insertion behavior.
/// This single-wire suggestion rejects it instead of silently dropping its edge.
/// Existing explicit replacement via `plan_material_graph_node_creation` remains
/// available to the host for a deliberate reconnect operation.
///
/// Nothing is published, no ID is allocated, and no graph is cloned. The host must
/// bind the popup, source/revision, catalog and current geometry, then pass this
/// candidate to the existing creation plan and one prepared domain transaction.
pub fn suggest_material_graph_autowire(
    graph: &MaterialGraphAsset,
    action: &GraphNodeDescriptor,
    node: &MaterialGraphNodeAsset,
    anchor: &GraphPortRef<String>,
    resolve_kind: impl FnMut(&AssetReference) -> Result<ResourceKind, String>,
) -> Result<Option<MaterialGraphLinkAsset>, Vec<MaterialGraphDiagnostic>> {
    // Reject stale/ambiguous source ports before resolving a new texture asset.
    let mut sources = graph.nodes.iter().filter(|node| node.id == anchor.node_id);
    let Some(source) = sources.next() else {
        return Err(vec![MaterialGraphDiagnostic::graph(
            "material graph autowire source node is no longer present",
        )]);
    };
    if source.id.trim().is_empty() || sources.next().is_some() {
        return Err(vec![MaterialGraphDiagnostic::graph(
            "material graph autowire source node identity is ambiguous",
        )]);
    }
    let valid_pin = match anchor.direction {
        GraphPinDirection::Input => graph_contract::has_input_pin(&source.kind, &anchor.pin_name),
        GraphPinDirection::Output => {
            graph_contract::output_pin(&source.kind) == Some(anchor.pin_name.as_str())
        }
    };
    if !valid_pin {
        return Err(vec![MaterialGraphDiagnostic::node(
            &source.id,
            "material graph autowire source pin or direction is no longer valid",
        )]);
    }
    if anchor.direction == GraphPinDirection::Input
        && graph
            .links
            .iter()
            .any(|edge| edge.to_node == anchor.node_id && edge.to_pin == anchor.pin_name)
    {
        return Err(vec![MaterialGraphDiagnostic::input(
            &source.id,
            &anchor.pin_name,
            "material graph autowire from an occupied input requires a multi-edge insertion plan",
        )]);
    }

    let diagnostics = validate_material_graph_node_creation(graph, action, node, resolve_kind);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let opposite_pins = match anchor.direction {
        GraphPinDirection::Input => action.outputs(),
        GraphPinDirection::Output => action.inputs(),
    };
    for pin in opposite_pins {
        let link = match anchor.direction {
            GraphPinDirection::Input => MaterialGraphLinkAsset {
                from_node: node.id.clone(),
                from_pin: pin.name().into(),
                to_node: source.id.clone(),
                to_pin: anchor.pin_name.clone(),
            },
            GraphPinDirection::Output => MaterialGraphLinkAsset {
                from_node: source.id.clone(),
                from_pin: anchor.pin_name.clone(),
                to_node: node.id.clone(),
                to_pin: pin.name().into(),
            },
        };
        if can_connect_material_graph_with_node(graph, &link, node).is_ok() {
            return Ok(Some(link));
        }
    }
    Ok(None)
}
