use zircon_runtime_interface::resource::ResourceKind;

use super::{
    plan_material_graph_node_creation, AssetReference, MaterialGraphAsset, MaterialGraphDiagnostic,
    MaterialGraphLinkAsset, MaterialGraphNodeAsset, MaterialGraphNodeCreationPlan,
    MaterialGraphNodeSearch,
};

/// Preflights a visible action against its typed node and optional autowire.
///
/// A row callback must carry its stable action key, rather than reinterpreting a
/// stale row index or the current keyboard selection. Hidden rows and closed
/// menus are rejected before any resource lookup. A visible mouse target need
/// not be the keyboard-selected row.
///
/// This borrows the existing creation plan and leaves both graph and menu alone,
/// including on failure. The host must retain its real popup/document incarnation,
/// measured graph-local anchor and source revision, then prepare and commit one
/// domain delta under the original retention permit/history. Only the accepted
/// activation may dismiss the popup and restore focus. This preflight is neither
/// a revision proof nor a document mutation permit.
pub fn plan_material_graph_node_search_creation<'a>(
    search: &MaterialGraphNodeSearch,
    graph: &'a MaterialGraphAsset,
    action_id: &str,
    candidate: &'a MaterialGraphNodeAsset,
    autowire: Option<&'a MaterialGraphLinkAsset>,
    resolve_kind: impl FnMut(&AssetReference) -> Result<ResourceKind, String>,
) -> Result<MaterialGraphNodeCreationPlan<'a>, Vec<MaterialGraphDiagnostic>> {
    if !search.is_open() {
        return Err(vec![MaterialGraphDiagnostic::graph(
            "material graph node search is closed",
        )]);
    }
    let Some(action) = search
        .matches()
        .iter()
        .find(|action| action.id() == action_id)
    else {
        return Err(vec![MaterialGraphDiagnostic::graph(format!(
            "material graph action `{action_id}` is not visible in the current node search"
        ))]);
    };
    plan_material_graph_node_creation(graph, action, candidate, autowire, resolve_kind)
}
