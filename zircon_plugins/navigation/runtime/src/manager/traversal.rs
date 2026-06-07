use std::borrow::Cow;

use zircon_runtime::asset::{NavMeshAsset, NavMeshLinkAsset};
use zircon_runtime::core::framework::navigation::{NavLinkTraversalMode, NavMeshAgentDescriptor};

/// Runtime agent ticks can consume only links that the agent may cross without
/// a gameplay handoff. Explicit path queries still see the original asset.
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

fn query_asset_with_links(
    asset: &NavMeshAsset,
    off_mesh_links: Vec<NavMeshLinkAsset>,
) -> Cow<'_, NavMeshAsset> {
    let mut filtered = asset.clone();
    filtered.off_mesh_links = off_mesh_links;
    Cow::Owned(filtered)
}
