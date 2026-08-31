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
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::{migrated_v1_link_id, validate_v1_link_count};

    #[test]
    fn optimization_batch_gq_runtime499_v1_link_ids_preserve_dense_checked_numbering() {
        assert_eq!(validate_v1_link_count(0), Ok(()));
        assert_eq!(validate_v1_link_count(u32::MAX as usize), Ok(()));
        assert_eq!(migrated_v1_link_id(0), 1);
        assert_eq!(migrated_v1_link_id(41), 42);
        assert_eq!(migrated_v1_link_id(u32::MAX as usize - 1), u32::MAX);

        #[cfg(target_pointer_width = "64")]
        assert_eq!(
            validate_v1_link_count(u32::MAX as usize + 1),
            Err(u32::MAX as usize + 1)
        );
    }

    #[test]
    #[ignore = "release benchmark submitted to the validation coordinator"]
    fn optimization_batch_gq_runtime499_v1_link_id_preflight_benchmark() {
        const MARKER: &str = "RUNTIME499_V1_LINK_ID_PREFLIGHT_BENCH_V1";
        const SAMPLES: usize = 31;
        const ITERATIONS: usize = 100_000;

        let mut optimized_samples = Vec::with_capacity(SAMPLES);
        let mut legacy_samples = Vec::with_capacity(SAMPLES);
        for _ in 0..SAMPLES {
            let started_at = Instant::now();
            let mut checksum = 0_u64;
            for index in 0..ITERATIONS {
                checksum = checksum.wrapping_add(u64::from(migrated_v1_link_id(black_box(index))));
            }
            black_box(checksum);
            optimized_samples.push(started_at.elapsed().as_nanos() / ITERATIONS as u128);

            let started_at = Instant::now();
            let mut checksum = 0_u64;
            for index in 0..ITERATIONS {
                let id = u32::try_from(black_box(index) + 1).expect("fixture link id");
                checksum = checksum.wrapping_add(u64::from(id));
            }
            black_box(checksum);
            legacy_samples.push(started_at.elapsed().as_nanos() / ITERATIONS as u128);
        }

        let optimized_p95_ns = p95(&mut optimized_samples);
        let legacy_p95_ns = p95(&mut legacy_samples);
        eprintln!(
            "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.90"
        );
        assert!(optimized_p95_ns <= legacy_p95_ns * 90 / 100);
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[samples.len().saturating_mul(95).div_ceil(100) - 1]
    }
}
