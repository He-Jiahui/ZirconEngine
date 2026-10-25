use std::collections::BTreeSet;

use zircon_editor::core::editor_authoring_extension::GraphNodeDescriptor;
use zircon_runtime_interface::resource::ResourceKind;

use super::{
    AssetReference, MaterialGraphAsset, MaterialGraphDiagnostic, MaterialGraphNodeAsset,
    MaterialGraphNodeKindAsset, MaterialGraphParameterAsset,
};

/// Checks a typed node against the current registered material action and graph.
///
/// This permits incomplete authoring graphs, but rejects ambiguous/reserved IDs,
/// invalid values, incompatible parameter keys and unresolved/non-texture assets.
/// The resolver must use the caller's current catalog and the complete reference;
/// a locator's extension does not establish its resource kind or UUID identity.
///
/// This function does not allocate an ID, mutate the graph, install a node or grant
/// a command permit. The host must bind the result and resolver to the same real
/// document/revision, then prepare one domain delta with its inverse in the
/// existing history. Positions are already in graph-local logical coordinates.
/// New-candidate errors target the graph, never a nonexistent source node.
pub fn validate_material_graph_node_creation(
    graph: &MaterialGraphAsset,
    action: &GraphNodeDescriptor,
    candidate: &MaterialGraphNodeAsset,
    mut resolve_kind: impl FnMut(&AssetReference) -> Result<ResourceKind, String>,
) -> Vec<MaterialGraphDiagnostic> {
    let palette = super::plugin::material_node_palette();
    let Some(registered) = palette.nodes().iter().find(|node| node.id() == action.id()) else {
        return vec![MaterialGraphDiagnostic::graph(format!(
            "material graph action `{}` is not registered",
            action.id()
        ))];
    };
    if registered != action {
        return vec![MaterialGraphDiagnostic::graph(format!(
            "material graph action `{}` no longer matches the registered descriptor",
            action.id()
        ))];
    }
    if action.id() != action_key(&candidate.kind) {
        return vec![MaterialGraphDiagnostic::graph(format!(
            "material graph action `{}` cannot create a `{}` node",
            action.id(),
            action_key(&candidate.kind)
        ))];
    }

    let mut ids = BTreeSet::new();
    for node in &graph.nodes {
        if node.id.trim().is_empty() || !ids.insert(node.id.as_str()) {
            return vec![MaterialGraphDiagnostic::graph(
                "material graph node creation requires unambiguous existing node IDs",
            )];
        }
    }
    let mut diagnostics = Vec::new();
    if candidate.id.trim().is_empty() {
        diagnostics.push(MaterialGraphDiagnostic::graph(
            "material graph new node id must not be empty",
        ));
    } else if ids.contains(candidate.id.as_str()) {
        diagnostics.push(MaterialGraphDiagnostic::graph(format!(
            "material graph node `{}` already exists",
            candidate.id
        )));
    } else if graph
        .links
        .iter()
        .any(|link| link.from_node == candidate.id || link.to_node == candidate.id)
    {
        diagnostics.push(MaterialGraphDiagnostic::graph(format!(
            "material graph new node id `{}` is already referenced by an existing edge",
            candidate.id
        )));
    }
    if !candidate.position.iter().all(|value| value.is_finite()) {
        diagnostics.push(MaterialGraphDiagnostic::graph(
            "material graph new node position must be finite graph-local coordinates",
        ));
    }
    match &candidate.kind {
        MaterialGraphNodeKindAsset::Output => {
            if graph
                .nodes
                .iter()
                .any(|node| matches!(node.kind, MaterialGraphNodeKindAsset::Output))
            {
                diagnostics.push(MaterialGraphDiagnostic::graph(
                    "material graph already contains an output node",
                ));
            }
        }
        MaterialGraphNodeKindAsset::ScalarParameter { name, default } => {
            validate_parameter(graph, name, true, default.is_finite(), &mut diagnostics);
        }
        MaterialGraphNodeKindAsset::VectorParameter { name, default } => {
            validate_parameter(
                graph,
                name,
                false,
                default.iter().all(|value| value.is_finite()),
                &mut diagnostics,
            );
        }
        _ => {}
    }
    // Do not resolve resources for an invalid or stale creation request.
    if diagnostics.is_empty() {
        if let MaterialGraphNodeKindAsset::TextureSample { texture } = &candidate.kind {
            match resolve_kind(texture) {
                Ok(ResourceKind::Texture) => {}
                Ok(kind) => diagnostics.push(MaterialGraphDiagnostic::graph(format!(
                    "material graph texture sample `{}` references {kind:?}, expected Texture",
                    candidate.id
                ))),
                Err(error) => diagnostics.push(MaterialGraphDiagnostic::graph(format!(
                    "material graph texture sample `{}` could not resolve its reference: {error}",
                    candidate.id
                ))),
            }
        }
    }
    diagnostics
}

pub(super) fn action_key(kind: &MaterialGraphNodeKindAsset) -> &'static str {
    match kind {
        MaterialGraphNodeKindAsset::Output => "output",
        MaterialGraphNodeKindAsset::TextureSample { .. } => "texture_sample",
        MaterialGraphNodeKindAsset::ScalarParameter { .. } => "scalar_parameter",
        MaterialGraphNodeKindAsset::VectorParameter { .. } => "vector_parameter",
        MaterialGraphNodeKindAsset::Add => "add",
        MaterialGraphNodeKindAsset::Multiply => "multiply",
    }
}

fn validate_parameter(
    graph: &MaterialGraphAsset,
    name: &str,
    scalar: bool,
    finite_default: bool,
    diagnostics: &mut Vec<MaterialGraphDiagnostic>,
) {
    if name.trim().is_empty() {
        diagnostics.push(MaterialGraphDiagnostic::graph(
            "material graph new parameter name must not be empty",
        ));
    }
    if !finite_default {
        diagnostics.push(MaterialGraphDiagnostic::graph(format!(
            "material graph new parameter `{name}` has a non-finite default"
        )));
    }
    for node in &graph.nodes {
        let (existing_name, existing_scalar) = match &node.kind {
            MaterialGraphNodeKindAsset::ScalarParameter { name, .. } => (name, true),
            MaterialGraphNodeKindAsset::VectorParameter { name, .. } => (name, false),
            _ => continue,
        };
        if existing_name == name && existing_scalar != scalar {
            diagnostics.push(MaterialGraphDiagnostic::node(
                &node.id,
                format!("material graph parameter `{name}` already has a different value type"),
            ));
        }
    }
    if let Some(value) = graph.parameters.get(name) {
        let (existing_scalar, finite) = match value {
            MaterialGraphParameterAsset::Scalar(value) => (true, value.is_finite()),
            MaterialGraphParameterAsset::Vector(value) => {
                (false, value.iter().all(|value| value.is_finite()))
            }
        };
        if existing_scalar != scalar {
            diagnostics.push(MaterialGraphDiagnostic::parameter(
                name,
                format!("material graph parameter `{name}` has an incompatible override type"),
            ));
        }
        if !finite {
            diagnostics.push(MaterialGraphDiagnostic::parameter(
                name,
                format!("material graph parameter `{name}` has a non-finite override"),
            ));
        }
    }
}
