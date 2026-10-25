use zircon_editor::core::editor_authoring_extension::GraphNodeDescriptor;
use zircon_runtime_interface::resource::ResourceKind;

use super::{
    connections::can_connect_material_graph_with_node, validate_material_graph_node_creation,
    AssetReference, MaterialGraphAsset, MaterialGraphConnectionPlan, MaterialGraphDiagnostic,
    MaterialGraphLinkAsset, MaterialGraphNodeAsset,
};

/// Borrowed node/edge proposal, checked together against one source graph.
///
/// This owns no document, snapshot or history. It is not a prepared command or
/// revision permit. Under its existing document lease and retention budget, the
/// host must prepare one delta containing the node insertion, optional old-edge
/// removal and new-edge insertion, with one inverse in the primary history.
/// Copying these references does not authorize a later or different revision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialGraphNodeCreationPlan<'a> {
    node: &'a MaterialGraphNodeAsset,
    link: Option<&'a MaterialGraphLinkAsset>,
    connection: Option<MaterialGraphConnectionPlan<'a>>,
}

impl<'a> MaterialGraphNodeCreationPlan<'a> {
    pub fn node(&self) -> &'a MaterialGraphNodeAsset {
        self.node
    }

    pub fn link(&self) -> Option<&'a MaterialGraphLinkAsset> {
        self.link
    }

    pub fn connection(&self) -> Option<MaterialGraphConnectionPlan<'a>> {
        self.connection
    }
}

/// Checks creation and optional autowire before publishing either change.
///
/// An explicit wire must join the new node to an existing endpoint. Pin keys and
/// value legality use the same analyzer as `can_connect_material_graph`; occupied
/// inputs are replaced before cycle/type checks. Missing authoring inputs remain
/// allowed. Runtime node/edge values are borrowed, without cloning the graph.
/// The host supplies an already measured graph-local position and resource resolver.
pub fn plan_material_graph_node_creation<'a>(
    graph: &'a MaterialGraphAsset,
    action: &GraphNodeDescriptor,
    node: &'a MaterialGraphNodeAsset,
    autowire: Option<&'a MaterialGraphLinkAsset>,
    resolve_kind: impl FnMut(&AssetReference) -> Result<ResourceKind, String>,
) -> Result<MaterialGraphNodeCreationPlan<'a>, Vec<MaterialGraphDiagnostic>> {
    if let Some(link) = autowire {
        let from_new = link.from_node == node.id;
        let to_new = link.to_node == node.id;
        if from_new == to_new {
            return Err(vec![MaterialGraphDiagnostic::graph(
                "material graph autowire must join the new node to an existing endpoint",
            )]);
        }
    }
    let diagnostics = validate_material_graph_node_creation(graph, action, node, resolve_kind);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let connection = autowire
        .map(|link| can_connect_material_graph_with_node(graph, link, node))
        .transpose()
        // A new node/pin is not yet a source location in the actual document.
        .map_err(|message| vec![MaterialGraphDiagnostic::graph(message)])?;
    Ok(MaterialGraphNodeCreationPlan {
        node,
        link: autowire,
        connection,
    })
}
