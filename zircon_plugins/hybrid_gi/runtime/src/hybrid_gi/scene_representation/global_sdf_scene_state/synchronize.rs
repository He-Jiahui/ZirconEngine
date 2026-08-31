use std::collections::BTreeSet;

use zircon_runtime::core::framework::render::RenderMeshBounds;
use zircon_runtime::core::math::Vec3;

use super::declarations::{
    HybridGiGlobalSdfClipmapBounds, HybridGiGlobalSdfPageKey, HybridGiGlobalSdfResidentPage,
    HybridGiGlobalSdfSceneState, GLOBAL_SDF_CLIPMAP_COUNT, GLOBAL_SDF_MAX_RESIDENT_PAGE_COUNT,
    GLOBAL_SDF_PAGES_PER_EDGE,
};

impl HybridGiGlobalSdfSceneState {
    pub(in crate::hybrid_gi) fn clipmap_bounds_for_camera(
        camera_position: Vec3,
    ) -> Vec<HybridGiGlobalSdfClipmapBounds> {
        let camera_position = if camera_position.is_finite() {
            camera_position
        } else {
            Vec3::ZERO
        };
        (0..GLOBAL_SDF_CLIPMAP_COUNT)
            .map(|level| HybridGiGlobalSdfClipmapBounds::for_camera(level as u32, camera_position))
            .collect()
    }

    pub(in crate::hybrid_gi) fn synchronize(
        &mut self,
        camera_position: Vec3,
        dirty_regions: &[RenderMeshBounds],
        page_budget: usize,
    ) -> bool {
        let next_clipmap_bounds = Self::clipmap_bounds_for_camera(camera_position);
        let desired_pages = desired_page_keys(
            &next_clipmap_bounds,
            page_budget.min(GLOBAL_SDF_MAX_RESIDENT_PAGE_COUNT),
        );
        let residency_changed =
            self.resident_pages.keys().copied().collect::<BTreeSet<_>>() != desired_pages;
        let next_generation = self.generation.saturating_add(1).max(1);
        self.clipmap_bounds = next_clipmap_bounds;

        self.resident_pages
            .retain(|key, _| desired_pages.contains(key));
        self.dirty_pages.retain(|key| desired_pages.contains(key));

        let mut pages_to_rebuild = BTreeSet::new();
        for key in desired_pages {
            if !self.resident_pages.contains_key(&key) {
                let Some(atlas_slot) = first_available_atlas_slot(self.resident_pages.values())
                else {
                    continue;
                };
                self.resident_pages.insert(
                    key,
                    HybridGiGlobalSdfResidentPage {
                        generation: self.generation,
                        initialized: false,
                        atlas_slot,
                    },
                );
                pages_to_rebuild.insert(key);
            }
        }

        for region in dirty_regions.iter().copied() {
            let intersecting_pages = self
                .resident_pages
                .keys()
                .copied()
                .filter(|key| {
                    self.page_influence_bounds(*key)
                        .is_some_and(|influence_bounds| aabb_intersects(influence_bounds, region))
                })
                .collect::<Vec<_>>();
            for key in intersecting_pages {
                pages_to_rebuild.insert(key);
            }
        }

        self.invalidate_pages(pages_to_rebuild, next_generation);
        residency_changed
    }

    pub(super) fn invalidate_pages(
        &mut self,
        pages_to_rebuild: BTreeSet<HybridGiGlobalSdfPageKey>,
        next_generation: u64,
    ) {
        if pages_to_rebuild.is_empty() {
            return;
        }
        self.generation = next_generation;
        for key in pages_to_rebuild {
            if let Some(page) = self.resident_pages.get_mut(&key) {
                page.generation = next_generation;
                page.initialized = false;
                self.dirty_pages.insert(key);
            }
        }
    }
}

fn first_available_atlas_slot<'a>(
    pages: impl IntoIterator<Item = &'a HybridGiGlobalSdfResidentPage>,
) -> Option<u32> {
    let occupied = pages
        .into_iter()
        .map(|page| page.atlas_slot)
        .collect::<BTreeSet<_>>();
    (0..GLOBAL_SDF_MAX_RESIDENT_PAGE_COUNT as u32).find(|slot| !occupied.contains(slot))
}

fn desired_page_keys(
    clipmaps: &[HybridGiGlobalSdfClipmapBounds],
    page_budget: usize,
) -> BTreeSet<HybridGiGlobalSdfPageKey> {
    if page_budget == 0 || clipmaps.is_empty() {
        return BTreeSet::new();
    }
    let budget_per_clipmap = page_budget / clipmaps.len();
    let remainder = page_budget % clipmaps.len();
    let mut desired = BTreeSet::new();
    for (index, clipmap) in clipmaps.iter().copied().enumerate() {
        let clipmap_budget = budget_per_clipmap + usize::from(index < remainder);
        let origin = clipmap.page_coordinate_origin();
        let mut candidates = Vec::with_capacity(
            (GLOBAL_SDF_PAGES_PER_EDGE * GLOBAL_SDF_PAGES_PER_EDGE * GLOBAL_SDF_PAGES_PER_EDGE)
                as usize,
        );
        for z in 0..GLOBAL_SDF_PAGES_PER_EDGE {
            for y in 0..GLOBAL_SDF_PAGES_PER_EDGE {
                for x in 0..GLOBAL_SDF_PAGES_PER_EDGE {
                    let centered_offset = [
                        x - GLOBAL_SDF_PAGES_PER_EDGE / 2,
                        y - GLOBAL_SDF_PAGES_PER_EDGE / 2,
                        z - GLOBAL_SDF_PAGES_PER_EDGE / 2,
                    ];
                    let distance_squared = centered_offset[0] * centered_offset[0]
                        + centered_offset[1] * centered_offset[1]
                        + centered_offset[2] * centered_offset[2];
                    candidates.push((distance_squared, [x, y, z], centered_offset));
                }
            }
        }
        candidates.sort_unstable_by_key(|(distance_squared, _, centered_offset)| {
            (
                *distance_squared,
                centered_offset[2],
                centered_offset[1],
                centered_offset[0],
            )
        });
        desired.extend(candidates.into_iter().take(clipmap_budget).map(
            |(_, local_coordinate, _)| HybridGiGlobalSdfPageKey {
                clipmap_id: clipmap.clipmap_id,
                page_coordinate: [
                    origin[0] + local_coordinate[0],
                    origin[1] + local_coordinate[1],
                    origin[2] + local_coordinate[2],
                ],
            },
        ));
    }
    desired
}

pub(super) fn aabb_intersects(left: RenderMeshBounds, right: RenderMeshBounds) -> bool {
    let left_min = Vec3::from_array(left.min);
    let left_max = Vec3::from_array(left.max);
    let right_min = Vec3::from_array(right.min);
    let right_max = Vec3::from_array(right.max);
    left_min.is_finite()
        && left_max.is_finite()
        && right_min.is_finite()
        && right_max.is_finite()
        && left_max.cmpge(right_min).all()
        && left_min.cmple(right_max).all()
}
