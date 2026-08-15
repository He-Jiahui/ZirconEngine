use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::core::framework::render::RenderMeshBounds;

use super::declarations::{
    HybridGiGlobalSdfClipmapBounds, HybridGiGlobalSdfPageKey, HybridGiGlobalSdfSceneState,
};
use super::synchronize::aabb_intersects;
use crate::hybrid_gi::scene_representation::HybridGiMeshSdfObject;

pub(in crate::hybrid_gi) const GLOBAL_SDF_MAX_PAGE_CANDIDATES: usize = 32;
const GLOBAL_SDF_MAX_PAGES_PER_OBJECT_PER_CLIPMAP: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::hybrid_gi) struct GlobalSdfInfluenceInput {
    pub(in crate::hybrid_gi) stable_instance_key: u64,
    pub(in crate::hybrid_gi) bounds: RenderMeshBounds,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::hybrid_gi) struct GlobalSdfInfluenceIndex {
    page_candidate_keys: BTreeMap<HybridGiGlobalSdfPageKey, Vec<u64>>,
    candidate_overflow_pages: BTreeSet<HybridGiGlobalSdfPageKey>,
    voxel_fallback_clipmaps: BTreeSet<u32>,
}

impl GlobalSdfInfluenceIndex {
    pub(super) fn rebuild(
        &mut self,
        clipmaps: &[HybridGiGlobalSdfClipmapBounds],
        resident_page_keys: &BTreeSet<HybridGiGlobalSdfPageKey>,
        inputs: impl IntoIterator<Item = GlobalSdfInfluenceInput>,
    ) -> BTreeSet<u32> {
        let previous_voxel_fallback_clipmaps = std::mem::take(&mut self.voxel_fallback_clipmaps);
        // Drop only evicted pages. A resident page can have no candidates this frame and
        // still needs to retain its bounded vector allocation for the next rebuild.
        self.page_candidate_keys
            .retain(|key, _| resident_page_keys.contains(key));
        for candidates in self.page_candidate_keys.values_mut() {
            candidates.clear();
        }
        self.candidate_overflow_pages.clear();

        for input in inputs {
            for clipmap in clipmaps.iter().copied() {
                let clipmap_id = clipmap.clipmap_id();
                let Some((page_min, page_max)) =
                    influenced_page_coordinate_range(clipmap, input.bounds)
                else {
                    continue;
                };
                let Some(page_count) = page_count_in_range(page_min, page_max) else {
                    self.voxel_fallback_clipmaps.insert(clipmap_id);
                    continue;
                };
                if page_count > GLOBAL_SDF_MAX_PAGES_PER_OBJECT_PER_CLIPMAP {
                    self.voxel_fallback_clipmaps.insert(clipmap_id);
                    continue;
                }
                for z in page_min[2]..=page_max[2] {
                    for y in page_min[1]..=page_max[1] {
                        for x in page_min[0]..=page_max[0] {
                            let key = HybridGiGlobalSdfPageKey {
                                clipmap_id,
                                page_coordinate: [x, y, z],
                            };
                            if resident_page_keys.contains(&key) {
                                self.insert_page_candidate(key, input.stable_instance_key);
                            }
                        }
                    }
                }
            }
        }
        for candidates in self.page_candidate_keys.values_mut() {
            candidates.sort_unstable();
            candidates.dedup();
        }
        previous_voxel_fallback_clipmaps
            .symmetric_difference(&self.voxel_fallback_clipmaps)
            .copied()
            .collect()
    }

    pub(super) fn page_candidate_keys(&self, key: HybridGiGlobalSdfPageKey) -> Option<&[u64]> {
        (!self.page_has_candidate_overflow(key))
            .then(|| self.page_candidate_keys.get(&key).map(Vec::as_slice))
            .flatten()
    }

    pub(super) fn page_has_candidate_overflow(&self, key: HybridGiGlobalSdfPageKey) -> bool {
        self.candidate_overflow_pages.contains(&key)
    }

    pub(super) fn clipmap_uses_voxel_fallback(&self, clipmap_id: u32) -> bool {
        self.voxel_fallback_clipmaps.contains(&clipmap_id)
    }

    pub(super) fn candidate_contributor_count(&self) -> usize {
        self.page_candidate_keys
            .iter()
            // A clipmap-level typed fallback prevents every one of its page
            // lists from becoming a sampleable Global SDF page.
            .filter(|(key, _)| !self.voxel_fallback_clipmaps.contains(&key.clipmap_id()))
            .map(|(_, candidates)| candidates.len())
            .sum()
    }

    pub(super) fn voxel_fallback_clipmap_count(&self) -> usize {
        self.voxel_fallback_clipmaps.len()
    }

    pub(super) fn candidate_bucket_capacity_bytes(&self) -> u64 {
        self.page_candidate_keys
            .values()
            .map(|candidates| {
                u64::try_from(candidates.capacity())
                    .unwrap_or(u64::MAX)
                    .saturating_mul(std::mem::size_of::<u64>() as u64)
            })
            .fold(0_u64, u64::saturating_add)
    }

    fn insert_page_candidate(&mut self, key: HybridGiGlobalSdfPageKey, stable_instance_key: u64) {
        if self.candidate_overflow_pages.contains(&key) {
            return;
        }
        if self
            .page_candidate_keys
            .get(&key)
            .is_some_and(|candidates| candidates.len() == GLOBAL_SDF_MAX_PAGE_CANDIDATES)
        {
            self.page_candidate_keys.remove(&key);
            self.candidate_overflow_pages.insert(key);
            return;
        }
        self.page_candidate_keys
            .entry(key)
            .or_default()
            .push(stable_instance_key);
    }
}

impl HybridGiGlobalSdfSceneState {
    pub(in crate::hybrid_gi) fn synchronize_influence(
        &mut self,
        objects: &[HybridGiMeshSdfObject],
    ) {
        let resident_page_keys = self.resident_pages.keys().copied().collect::<BTreeSet<_>>();
        let fallback_changes = self.influence_index.rebuild(
            &self.clipmap_bounds,
            &resident_page_keys,
            objects
                .iter()
                .filter(|object| object.participates_in_global_sdf())
                .map(|object| GlobalSdfInfluenceInput {
                    stable_instance_key: object.stable_instance_key(),
                    bounds: object.bounds(),
                }),
        );
        if fallback_changes.is_empty() {
            return;
        }
        let affected_pages = self
            .resident_pages
            .keys()
            .filter(|key| fallback_changes.contains(&key.clipmap_id))
            .copied()
            .collect::<BTreeSet<_>>();
        self.invalidate_pages(affected_pages, self.generation.saturating_add(1).max(1));
    }
}

fn influenced_page_coordinate_range(
    clipmap: HybridGiGlobalSdfClipmapBounds,
    bounds: RenderMeshBounds,
) -> Option<([i32; 3], [i32; 3])> {
    let page_world_size = clipmap.page_world_size();
    if !page_world_size.is_finite() || page_world_size <= 0.0 {
        return None;
    }
    let clipmap_bounds = clipmap.world_bounds();
    let expanded_clipmap_bounds = RenderMeshBounds::from_min_max(
        clipmap_bounds.min.map(|value| value - page_world_size),
        clipmap_bounds.max.map(|value| value + page_world_size),
    );
    if !aabb_intersects(expanded_clipmap_bounds, bounds) {
        return None;
    }
    let mut page_min = [0; 3];
    let mut page_max = [0; 3];
    for axis in 0..3 {
        page_min[axis] = finite_floor_to_i32(bounds.min[axis] / page_world_size - 2.0)?;
        page_max[axis] = finite_floor_to_i32(bounds.max[axis] / page_world_size + 1.0)?;
    }
    Some((page_min, page_max))
}

fn page_count_in_range(page_min: [i32; 3], page_max: [i32; 3]) -> Option<usize> {
    page_min
        .into_iter()
        .zip(page_max)
        .try_fold(1_usize, |count, (min, max)| {
            let axis_count = usize::try_from(i64::from(max) - i64::from(min) + 1).ok()?;
            count.checked_mul(axis_count)
        })
}

fn finite_floor_to_i32(value: f32) -> Option<i32> {
    (value.is_finite() && value >= i32::MIN as f32 && value <= i32::MAX as f32)
        .then(|| value.floor() as i32)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use zircon_runtime::core::framework::render::RenderMeshBounds;
    use zircon_runtime::core::math::Vec3;

    use super::{GlobalSdfInfluenceIndex, GlobalSdfInfluenceInput};
    use crate::hybrid_gi::scene_representation::{
        HybridGiGlobalSdfClipmapBounds, HybridGiGlobalSdfPageKey,
    };

    #[test]
    fn influence_index_keeps_objects_inside_the_adjacent_page_influence_band() {
        let clipmap = HybridGiGlobalSdfClipmapBounds::new(0, Vec3::ZERO, 16.0);
        let page_key = HybridGiGlobalSdfPageKey {
            clipmap_id: 0,
            page_coordinate: [0, 0, 0],
        };
        let mut index = GlobalSdfInfluenceIndex::default();
        index.rebuild(
            &[clipmap],
            &BTreeSet::from([page_key]),
            [GlobalSdfInfluenceInput {
                stable_instance_key: 7,
                bounds: RenderMeshBounds::from_min_max([4.25, 0.0, 0.0], [4.5, 0.25, 0.25]),
            }],
        );

        assert_eq!(index.page_candidate_keys(page_key), Some(&[7][..]));
        assert!(!index.page_has_candidate_overflow(page_key));
        assert_eq!(index.candidate_contributor_count(), 1);
    }

    #[test]
    fn influence_index_marks_a_page_when_the_bounded_candidate_list_overflows() {
        let clipmap = HybridGiGlobalSdfClipmapBounds::new(0, Vec3::ZERO, 16.0);
        let page_key = HybridGiGlobalSdfPageKey {
            clipmap_id: 0,
            page_coordinate: [0, 0, 0],
        };
        let inputs = (0..33)
            .map(|stable_instance_key| GlobalSdfInfluenceInput {
                stable_instance_key,
                bounds: RenderMeshBounds::from_min_max([0.0; 3], [0.25; 3]),
            })
            .collect::<Vec<_>>();
        let mut index = GlobalSdfInfluenceIndex::default();
        index.rebuild(&[clipmap], &BTreeSet::from([page_key]), inputs);

        assert!(index.page_has_candidate_overflow(page_key));
        assert_eq!(index.page_candidate_keys(page_key), None);
        assert_eq!(index.candidate_contributor_count(), 0);
    }

    #[test]
    fn oversized_object_promotes_its_clipmap_to_typed_fallback() {
        let clipmap = HybridGiGlobalSdfClipmapBounds::new(0, Vec3::ZERO, 16.0);
        let page_key = HybridGiGlobalSdfPageKey {
            clipmap_id: 0,
            page_coordinate: [0, 0, 0],
        };
        let mut index = GlobalSdfInfluenceIndex::default();
        index.rebuild(
            &[clipmap],
            &BTreeSet::from([page_key]),
            [GlobalSdfInfluenceInput {
                stable_instance_key: 9,
                bounds: RenderMeshBounds::from_min_max([-32.0; 3], [32.0; 3]),
            }],
        );

        assert!(index.clipmap_uses_voxel_fallback(0));
        assert_eq!(index.voxel_fallback_clipmap_count(), 1);
    }

    #[test]
    fn clipmap_fallback_excludes_other_page_candidates_from_materializable_count() {
        let clipmap = HybridGiGlobalSdfClipmapBounds::new(0, Vec3::ZERO, 16.0);
        let page_key = HybridGiGlobalSdfPageKey {
            clipmap_id: 0,
            page_coordinate: [0, 0, 0],
        };
        let mut index = GlobalSdfInfluenceIndex::default();
        index.rebuild(
            &[clipmap],
            &BTreeSet::from([page_key]),
            [
                GlobalSdfInfluenceInput {
                    stable_instance_key: 7,
                    bounds: RenderMeshBounds::from_min_max([0.0; 3], [0.25; 3]),
                },
                GlobalSdfInfluenceInput {
                    stable_instance_key: 9,
                    bounds: RenderMeshBounds::from_min_max([-32.0; 3], [32.0; 3]),
                },
            ],
        );

        assert_eq!(index.page_candidate_keys(page_key), Some(&[7][..]));
        assert!(index.clipmap_uses_voxel_fallback(0));
        assert_eq!(index.candidate_contributor_count(), 0);
    }

    #[test]
    fn resident_empty_page_keeps_its_candidate_vector_capacity() {
        let clipmap = HybridGiGlobalSdfClipmapBounds::new(0, Vec3::ZERO, 16.0);
        let page_key = HybridGiGlobalSdfPageKey {
            clipmap_id: 0,
            page_coordinate: [0, 0, 0],
        };
        let resident_pages = BTreeSet::from([page_key]);
        let mut index = GlobalSdfInfluenceIndex::default();
        index.rebuild(
            &[clipmap],
            &resident_pages,
            [GlobalSdfInfluenceInput {
                stable_instance_key: 7,
                bounds: RenderMeshBounds::from_min_max([0.0; 3], [0.25; 3]),
            }],
        );
        let capacity = index.page_candidate_keys[&page_key].capacity();

        index.rebuild(&[clipmap], &resident_pages, []);

        let candidates = &index.page_candidate_keys[&page_key];
        assert!(candidates.is_empty());
        assert_eq!(candidates.capacity(), capacity);
        assert_eq!(
            index.candidate_bucket_capacity_bytes(),
            u64::try_from(capacity).unwrap() * std::mem::size_of::<u64>() as u64
        );
    }
}
