//! Isolated reader for the retired version-one navmesh wire shape.

use serde::Deserialize;

use crate::core::math::Real;

use super::super::constants::NavAreaId;
use super::super::off_mesh_link::{NavLinkMotion, NavLinkTraversalMode};
use super::{
    NavMeshAreaCostAsset, NavMeshAsset, NavMeshLinkAsset, NavMeshLinkCapacity, NavMeshPolygonAsset,
    NavMeshTileAsset,
};

pub(super) const VERSION: u32 = 1;

#[derive(Deserialize)]
pub(super) struct NavMeshAssetV1 {
    version: u32,
    agent_type: String,
    settings_hash: u64,
    area_costs: Vec<NavMeshAreaCostAsset>,
    vertices: Vec<[Real; 3]>,
    indices: Vec<u32>,
    polygons: Vec<NavMeshPolygonAsset>,
    tiles: Vec<NavMeshTileAsset>,
    off_mesh_links: Vec<NavMeshLinkAssetV1>,
}

#[derive(Deserialize)]
struct NavMeshLinkAssetV1 {
    start: [Real; 3],
    end: [Real; 3],
    width: Real,
    bidirectional: bool,
    area: NavAreaId,
    cost_override: Option<Real>,
    traversal_mode: NavLinkTraversalMode,
}

impl NavMeshAssetV1 {
    /// 旧格式没有连接所有者和稳定 ID；迁移时保留路径几何并给连接分配非零 ID，供当前运行时消费。
    pub(super) fn migrate(self) -> Result<NavMeshAsset, usize> {
        debug_assert_eq!(self.version, VERSION);
        let link_count = self.off_mesh_links.len();
        validate_v1_link_count(link_count)?;
        let mut off_mesh_links = Vec::with_capacity(link_count);
        for (index, link) in self.off_mesh_links.into_iter().enumerate() {
            off_mesh_links.push(NavMeshLinkAsset {
                id: migrated_v1_link_id(index),
                owner_entity: 0,
                lane_index: 0,
                capacity: NavMeshLinkCapacity::Unbounded,
                motion: NavLinkMotion::Linear,
                arc_height: 0.0,
                start: link.start,
                end: link.end,
                width: link.width,
                bidirectional: link.bidirectional,
                area: link.area,
                cost_override: link.cost_override,
                traversal_mode: link.traversal_mode,
            });
        }
        Ok(NavMeshAsset {
            version: NavMeshAsset::VERSION,
            agent_type: self.agent_type,
            settings_hash: self.settings_hash,
            area_costs: self.area_costs,
            vertices: self.vertices,
            indices: self.indices,
            polygons: self.polygons,
            tiles: self.tiles,
            off_mesh_links,
        })
    }
}

fn validate_v1_link_count(link_count: usize) -> Result<(), usize> {
    if link_count > u32::MAX as usize {
        Err(link_count)
    } else {
        Ok(())
    }
}

fn migrated_v1_link_id(index: usize) -> u32 {
    debug_assert!(index < u32::MAX as usize);
    index as u32 + 1
}

#[cfg(test)]
#[path = "tests/v1.rs"]
mod tests;
