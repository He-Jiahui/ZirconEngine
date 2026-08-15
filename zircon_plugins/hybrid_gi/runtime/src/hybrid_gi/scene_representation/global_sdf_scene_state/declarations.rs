use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::core::framework::render::RenderMeshBounds;
use zircon_runtime::core::math::Vec3;

use super::influence::GlobalSdfInfluenceIndex;

pub(in crate::hybrid_gi) const GLOBAL_SDF_CLIPMAP_COUNT: usize = 4;
pub(in crate::hybrid_gi) const GLOBAL_SDF_MAX_RESIDENT_PAGE_COUNT: usize = 128;
pub(super) const GLOBAL_SDF_CLIPMAP_CELLS_PER_EDGE: i32 = 64;
pub(super) const GLOBAL_SDF_PAGE_CELLS_PER_EDGE: i32 = 8;
pub(in crate::hybrid_gi) const GLOBAL_SDF_PAGES_PER_EDGE: i32 =
    GLOBAL_SDF_CLIPMAP_CELLS_PER_EDGE / GLOBAL_SDF_PAGE_CELLS_PER_EDGE;
pub(super) const GLOBAL_SDF_BASE_CELL_SIZE: f32 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::hybrid_gi) struct HybridGiGlobalSdfClipmapBounds {
    pub(super) clipmap_id: u32,
    pub(super) center: Vec3,
    pub(super) half_extent: f32,
    pub(super) page_world_size: f32,
}

impl HybridGiGlobalSdfClipmapBounds {
    pub(in crate::hybrid_gi) fn new(clipmap_id: u32, center: Vec3, half_extent: f32) -> Self {
        let cell_size = (half_extent * 2.0) / GLOBAL_SDF_CLIPMAP_CELLS_PER_EDGE as f32;
        Self {
            clipmap_id,
            center,
            half_extent,
            page_world_size: cell_size * GLOBAL_SDF_PAGE_CELLS_PER_EDGE as f32,
        }
    }

    pub(super) fn for_camera(clipmap_id: u32, camera_position: Vec3) -> Self {
        let cell_size = GLOBAL_SDF_BASE_CELL_SIZE * 2.0_f32.powi(clipmap_id as i32);
        let page_world_size = cell_size * GLOBAL_SDF_PAGE_CELLS_PER_EDGE as f32;
        let center = (camera_position / page_world_size).round() * page_world_size;
        Self {
            clipmap_id,
            center,
            half_extent: cell_size * GLOBAL_SDF_CLIPMAP_CELLS_PER_EDGE as f32 * 0.5,
            page_world_size,
        }
    }

    pub(in crate::hybrid_gi) fn clipmap_id(self) -> u32 {
        self.clipmap_id
    }

    pub(in crate::hybrid_gi) fn page_world_size(self) -> f32 {
        self.page_world_size
    }

    /// Global page coordinate of this clipmap's deterministic 8^3 table origin.
    pub(in crate::hybrid_gi) fn page_coordinate_origin(self) -> [i32; 3] {
        let center = (self.center / self.page_world_size).round().as_ivec3();
        let half_page_count = GLOBAL_SDF_PAGES_PER_EDGE / 2;
        [
            center.x - half_page_count,
            center.y - half_page_count,
            center.z - half_page_count,
        ]
    }

    pub(in crate::hybrid_gi) fn intersects(self, bounds: RenderMeshBounds) -> bool {
        if !self.center.is_finite() || !self.half_extent.is_finite() || self.half_extent <= 0.0 {
            return false;
        }
        let clipmap_min = self.center - Vec3::splat(self.half_extent);
        let clipmap_max = self.center + Vec3::splat(self.half_extent);
        let bounds_min = Vec3::from_array(bounds.min);
        let bounds_max = Vec3::from_array(bounds.max);
        bounds_min.is_finite()
            && bounds_max.is_finite()
            && bounds_max.cmpge(clipmap_min).all()
            && bounds_min.cmple(clipmap_max).all()
    }

    pub(in crate::hybrid_gi) fn world_bounds(self) -> RenderMeshBounds {
        let half_extent = Vec3::splat(self.half_extent.max(0.0));
        RenderMeshBounds::from_min_max(
            (self.center - half_extent).to_array(),
            (self.center + half_extent).to_array(),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::hybrid_gi) struct HybridGiGlobalSdfPageKey {
    pub(super) clipmap_id: u32,
    pub(super) page_coordinate: [i32; 3],
}

impl HybridGiGlobalSdfPageKey {
    pub(in crate::hybrid_gi) fn clipmap_id(self) -> u32 {
        self.clipmap_id
    }

    pub(in crate::hybrid_gi) fn page_coordinate(self) -> [i32; 3] {
        self.page_coordinate
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::hybrid_gi) struct HybridGiGlobalSdfPageBuildRequest {
    pub(super) key: HybridGiGlobalSdfPageKey,
    pub(super) requested_generation: u64,
    pub(super) atlas_slot: u32,
}

impl HybridGiGlobalSdfPageBuildRequest {
    pub(in crate::hybrid_gi) fn key(self) -> HybridGiGlobalSdfPageKey {
        self.key
    }

    pub(in crate::hybrid_gi) fn requested_generation(self) -> u64 {
        self.requested_generation
    }

    pub(in crate::hybrid_gi) fn atlas_slot(self) -> u32 {
        self.atlas_slot
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct HybridGiGlobalSdfResidentPage {
    pub(super) generation: u64,
    pub(super) initialized: bool,
    pub(super) atlas_slot: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::hybrid_gi) struct HybridGiGlobalSdfSceneState {
    pub(super) clipmap_bounds: Vec<HybridGiGlobalSdfClipmapBounds>,
    pub(super) resident_pages: BTreeMap<HybridGiGlobalSdfPageKey, HybridGiGlobalSdfResidentPage>,
    pub(super) dirty_pages: BTreeSet<HybridGiGlobalSdfPageKey>,
    pub(super) influence_index: GlobalSdfInfluenceIndex,
    pub(super) generation: u64,
}
