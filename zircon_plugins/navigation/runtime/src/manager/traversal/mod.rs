mod advance;
mod capacity;
mod selection;
mod state;
#[cfg(test)]
mod tests;

use std::borrow::Cow;

use zircon_runtime::core::framework::navigation::{
    NavAgentTickReport, NavLinkTraversalMode, NavMeshAgentDescriptor, NavMeshHandle, NavPathResult,
    OffMeshTraverseEvent,
};
use zircon_runtime::core::framework::navigation::{NavMeshAsset, NavMeshLinkAsset};
use zircon_runtime::core::math::{Real, Vec3};

use super::DefaultNavigationManager;

pub(super) use advance::{ActiveTraversalStep, DirectTraversalPosition};
pub(super) use state::OffMeshTraversalRuntime;

/// Runtime agent ticks consume only links that may be crossed without a gameplay handoff.
/// Explicit path queries continue to see the immutable source asset.
pub(super) fn automatic_agent_query_asset<'a>(
    asset: &'a NavMeshAsset,
    agent: &NavMeshAgentDescriptor,
) -> Cow<'a, NavMeshAsset> {
    if asset.off_mesh_links.is_empty() {
        return Cow::Borrowed(asset);
    }
    if !agent.auto_traverse_links {
        return query_asset_with_links(asset, Vec::new());
    }

    let automatic_links = asset
        .off_mesh_links
        .iter()
        .filter(|link| matches!(link.traversal_mode, NavLinkTraversalMode::Automatic))
        .cloned()
        .collect::<Vec<_>>();
    if automatic_links.len() == asset.off_mesh_links.len() {
        Cow::Borrowed(asset)
    } else {
        query_asset_with_links(asset, automatic_links)
    }
}

pub(super) fn begin_from_path(
    manager: &DefaultNavigationManager,
    agent_entity: u64,
    nav_mesh: NavMeshHandle,
    asset: &NavMeshAsset,
    path: &NavPathResult,
    current: Vec3,
) -> Option<Vec3> {
    let link = selection::select_upcoming_link(asset, path)?;
    let mut runtime_state = manager.lock_state();
    runtime_state
        .off_mesh_traversal
        .begin(agent_entity, nav_mesh, link, current)
}

pub(super) fn advance_active(
    manager: &DefaultNavigationManager,
    agent_entity: u64,
    current: Vec3,
    agent: &NavMeshAgentDescriptor,
    dt_seconds: Real,
) -> Option<ActiveTraversalStep> {
    manager
        .lock_state()
        .off_mesh_traversal
        .advance(agent_entity, current, agent, dt_seconds)
}

pub(super) fn clear_agent(manager: &DefaultNavigationManager, agent_entity: u64) {
    manager
        .lock_state()
        .off_mesh_traversal
        .clear_agent(agent_entity);
}

pub(super) fn retain_agents(manager: &DefaultNavigationManager, active_agents: &[u64]) {
    manager
        .lock_state()
        .off_mesh_traversal
        .retain_agents(active_agents);
}

pub(super) fn update_report_metrics(
    manager: &DefaultNavigationManager,
    report: &mut NavAgentTickReport,
) {
    let runtime_state = manager.lock_state();
    report.traversing_agents = runtime_state.off_mesh_traversal.traversing_agents();
    report.queued_link_agents = runtime_state.off_mesh_traversal.queued_agents();
}

pub(super) fn record_event(report: &mut NavAgentTickReport, event: OffMeshTraverseEvent) {
    report.off_mesh_events.push(event);
}

fn query_asset_with_links(
    asset: &NavMeshAsset,
    off_mesh_links: Vec<NavMeshLinkAsset>,
) -> Cow<'_, NavMeshAsset> {
    let mut filtered = asset.clone();
    filtered.off_mesh_links = off_mesh_links;
    Cow::Owned(filtered)
}
