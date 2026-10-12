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
