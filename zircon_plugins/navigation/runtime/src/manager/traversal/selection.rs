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
mod tests {
    use zircon_runtime::core::framework::navigation::{
        NavLinkMotion, NavPathPoint, NavPathStatus, AREA_JUMP, AREA_WALKABLE,
    };
    use zircon_runtime::core::framework::navigation::{NavMeshLinkAsset, NavMeshLinkCapacity};

    use super::*;

    #[test]
    fn distant_link_is_not_selected_before_preceding_corner() {
        let asset = asset_with_link();
        let mut path = path_with_link_at(2);

        assert!(select_upcoming_link(&asset, &path).is_none());

        path.points[1].off_mesh_link_id = Some(7);
        path.points[2].off_mesh_link_id = None;
        assert_eq!(
            select_upcoming_link(&asset, &path).map(|link| link.id),
            Some(7)
        );
    }

    fn asset_with_link() -> NavMeshAsset {
        let mut asset = NavMeshAsset::simple_quad("humanoid", 8.0);
        asset.off_mesh_links.push(NavMeshLinkAsset {
            id: 7,
            owner_entity: 0,
            lane_index: 0,
            capacity: NavMeshLinkCapacity::Unbounded,
            motion: NavLinkMotion::Linear,
            arc_height: 0.0,
            start: [4.0, 0.0, 2.0],
            end: [6.0, 0.0, 2.0],
            width: 0.5,
            bidirectional: true,
            area: AREA_JUMP,
            cost_override: None,
            traversal_mode: Default::default(),
        });
        asset
    }

    fn path_with_link_at(index: usize) -> NavPathResult {
        let mut points = vec![
            NavPathPoint {
                position: [0.0, 0.0, 0.0],
                area: AREA_WALKABLE,
                off_mesh_link_id: None,
                flags: Vec::new(),
            },
            NavPathPoint {
                position: [2.0, 0.0, 2.0],
                area: AREA_WALKABLE,
                off_mesh_link_id: None,
                flags: Vec::new(),
            },
            NavPathPoint {
                position: [4.0, 0.0, 2.0],
                area: AREA_JUMP,
                off_mesh_link_id: None,
                flags: vec!["off_mesh_link".to_string()],
            },
        ];
        points[index].off_mesh_link_id = Some(7);
        NavPathResult {
            status: NavPathStatus::Complete,
            points,
            length: 6.0,
            visited_nodes: 3,
        }
    }
}
