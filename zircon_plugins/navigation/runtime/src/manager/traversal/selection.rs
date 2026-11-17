use zircon_runtime::core::framework::navigation::NavPathResult;
use zircon_runtime::core::framework::navigation::{NavMeshAsset, NavMeshLinkAsset};
use zircon_runtime::core::math::Vec3;

pub(super) fn select_upcoming_link<'a>(
    asset: &'a NavMeshAsset,
    path: &NavPathResult,
) -> Option<(&'a NavMeshLinkAsset, Vec3)> {
    let (point, link_id) = path
        .points
        .iter()
        .take(2)
        .find_map(|point| point.off_mesh_link_id.map(|id| (point, id)))?;
    let link = asset
        .off_mesh_links
        .iter()
        .find(|link| link.id == link_id)?;
    Some((link, Vec3::from_array(point.position)))
}

#[cfg(test)]
#[path = "tests/selection.rs"]
mod tests;
