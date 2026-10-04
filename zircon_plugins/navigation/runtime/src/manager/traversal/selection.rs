use zircon_runtime::core::framework::navigation::NavPathResult;
use zircon_runtime::core::framework::navigation::{NavMeshAsset, NavMeshLinkAsset};

pub(super) fn select_upcoming_link<'a>(
    asset: &'a NavMeshAsset,
    path: &NavPathResult,
) -> Option<&'a NavMeshLinkAsset> {
    let link_id = path
        .points
        .iter()
        .take(2)
        .find_map(|point| point.off_mesh_link_id)?;
    asset.off_mesh_links.iter().find(|link| link.id == link_id)
}

#[cfg(test)]
#[path = "tests/selection.rs"]
mod tests;
