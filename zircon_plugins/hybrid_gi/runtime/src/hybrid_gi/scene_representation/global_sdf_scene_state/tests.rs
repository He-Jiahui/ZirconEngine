use super::*;

#[test]
fn global_sdf_statistics_distinguish_dirty_and_sampleable_pages() {
    let mut state = HybridGiGlobalSdfSceneState::default();
    state.synchronize(Vec3::ZERO, &[], 1);

    assert_eq!(state.resident_page_count(), 1);
    assert_eq!(state.dirty_page_count(), 1);
    assert_eq!(state.sampleable_page_count(), 0);

    let requests = state.dirty_page_build_requests();
    state.commit_pages(&requests);

    assert_eq!(state.resident_page_count(), 1);
    assert_eq!(state.dirty_page_count(), 0);
    assert_eq!(state.sampleable_page_count(), 1);
}

#[test]
fn camera_motion_inside_one_page_keeps_residency_stable() {
    let mut state = HybridGiGlobalSdfSceneState::default();
    state.synchronize(Vec3::new(0.1, 0.0, 0.0), &[], 32);
    let initial_descriptors = state.clipmap_bounds().to_vec();
    let initial_pages = state.resident_page_keys();
    let initial_dirty = state.dirty_page_build_requests();
    state.commit_pages(&initial_dirty);

    state.synchronize(Vec3::new(1.9, 0.0, 0.0), &[], 32);

    assert_eq!(state.clipmap_bounds(), initial_descriptors);
    assert_eq!(state.resident_page_keys(), initial_pages);
    assert!(state.dirty_page_keys().is_empty());
}

#[test]
fn crossing_page_boundary_scrolls_with_bounded_deterministic_residency() {
    let mut state = HybridGiGlobalSdfSceneState::default();
    state.synchronize(Vec3::ZERO, &[], 17);
    let initial_pages = state.resident_page_keys();
    let initial_dirty = state.dirty_page_build_requests();
    state.commit_pages(&initial_dirty);

    state.synchronize(Vec3::new(4.1, 0.0, 0.0), &[], 17);

    assert_ne!(state.resident_page_keys(), initial_pages);
    assert_eq!(state.resident_page_count(), 17);
    assert!(!state.dirty_page_keys().is_empty());
    assert!(state.resident_page_count() <= 17);
}

#[test]
fn dirty_region_only_invalidates_intersecting_resident_pages() {
    let mut state = HybridGiGlobalSdfSceneState::default();
    state.synchronize(Vec3::ZERO, &[], 64);
    let initial_dirty = state.dirty_page_build_requests();
    state.commit_pages(&initial_dirty);
    let region = RenderMeshBounds::from_min_max([-0.25; 3], [0.25; 3]);

    state.synchronize(Vec3::ZERO, &[region], 64);

    let dirty = state.dirty_page_keys();
    assert!(!dirty.is_empty());
    assert!(dirty.len() < state.resident_page_count());
    assert!(dirty.iter().all(|key| {
        state
            .page_influence_bounds(*key)
            .is_some_and(|bounds| aabb_intersects(bounds, region))
    }));
}

#[test]
fn pages_are_not_sampleable_until_committed() {
    let mut state = HybridGiGlobalSdfSceneState::default();
    state.synchronize(Vec3::ZERO, &[], 8);
    let dirty = state.dirty_page_build_requests();
    let page = dirty[0].key;

    assert!(!state.is_page_sampleable(page));
    state.commit_pages(&[dirty[0]]);
    assert!(state.is_page_sampleable(page));
}

#[test]
fn terminal_fallback_page_stays_uninitialized_until_a_scene_change_redirties_it() {
    let mut state = HybridGiGlobalSdfSceneState::default();
    state.synchronize(Vec3::ZERO, &[], 1);
    let request = state.dirty_page_build_requests()[0];
    let bounds = state.page_bounds(request.key()).unwrap();

    state.resolve_pages_to_fallback(&[request]);

    assert!(state.dirty_page_keys().is_empty());
    assert!(!state.is_page_sampleable(request.key()));

    state.synchronize(Vec3::ZERO, &[bounds], 1);
    let retried = state.dirty_page_build_requests()[0];
    assert_eq!(retried.key(), request.key());
    assert_ne!(
        retried.requested_generation(),
        request.requested_generation()
    );
}

#[test]
fn adjacent_dirty_region_redirties_a_terminal_fallback_page_inside_its_sdf_influence_band() {
    let mut state = HybridGiGlobalSdfSceneState::default();
    state.synchronize(Vec3::ZERO, &[], 1);
    let request = state.dirty_page_build_requests()[0];
    let page_bounds = state.page_bounds(request.key()).unwrap();
    let influence_bounds = state.page_influence_bounds(request.key()).unwrap();
    let page_extent = page_bounds.max[0] - page_bounds.min[0];
    let center_y = (page_bounds.min[1] + page_bounds.max[1]) * 0.5;
    let center_z = (page_bounds.min[2] + page_bounds.max[2]) * 0.5;
    let adjacent_source = RenderMeshBounds::from_min_max(
        [
            page_bounds.max[0] + page_extent * 0.25,
            center_y - 0.25,
            center_z - 0.25,
        ],
        [
            page_bounds.max[0] + page_extent * 0.5,
            center_y + 0.25,
            center_z + 0.25,
        ],
    );
    assert!(!aabb_intersects(page_bounds, adjacent_source));
    assert!(aabb_intersects(influence_bounds, adjacent_source));

    state.resolve_pages_to_fallback(&[request]);
    state.synchronize(Vec3::ZERO, &[adjacent_source], 1);

    let retried = state.dirty_page_build_requests()[0];
    assert_eq!(retried.key(), request.key());
    assert_ne!(
        retried.requested_generation(),
        request.requested_generation()
    );
}

#[test]
fn evicted_and_reinserted_page_rejects_stale_completion() {
    let mut state = HybridGiGlobalSdfSceneState::default();
    state.synchronize(Vec3::ZERO, &[], 8);
    let stale = state.dirty_page_build_requests();
    let page = stale[0].key;

    state.synchronize(Vec3::new(64.0, 0.0, 0.0), &[], 8);
    state.synchronize(Vec3::ZERO, &[], 8);
    assert!(state.dirty_page_keys().contains(&page));
    state.commit_pages(&[stale[0]]);

    assert!(!state.is_page_sampleable(page));
}

#[test]
fn redirtied_in_flight_page_rejects_older_generation_completion() {
    let mut state = HybridGiGlobalSdfSceneState::default();
    state.synchronize(Vec3::ZERO, &[], 32);
    let initial = state.dirty_page_build_requests();
    state.commit_pages(&initial);
    let page = initial[0].key();
    let page_bounds = state.page_bounds(page).unwrap();

    state.synchronize(Vec3::ZERO, &[page_bounds], 32);
    let first_build = state
        .dirty_page_build_requests()
        .into_iter()
        .find(|request| request.key() == page)
        .unwrap();
    state.synchronize(Vec3::ZERO, &[page_bounds], 32);
    let second_build = state
        .dirty_page_build_requests()
        .into_iter()
        .find(|request| request.key() == page)
        .unwrap();

    assert_ne!(
        first_build.requested_generation(),
        second_build.requested_generation()
    );
    state.commit_pages(&[first_build]);
    assert!(!state.is_page_sampleable(page));
    state.commit_pages(&[second_build]);
    assert!(state.is_page_sampleable(page));
}

#[test]
fn resident_pages_keep_unique_stable_atlas_slots() {
    let mut state = HybridGiGlobalSdfSceneState::default();
    state.synchronize(Vec3::ZERO, &[], 32);
    let initial = state.dirty_page_build_requests();
    let initial_slots = initial
        .iter()
        .map(|request| (request.key(), request.atlas_slot()))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        initial_slots
            .values()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        initial_slots.len()
    );
    state.commit_pages(&initial);

    state.synchronize(Vec3::new(4.1, 0.0, 0.0), &[], 32);

    for request in state.dirty_page_build_requests() {
        if let Some(initial_slot) = initial_slots.get(&request.key()) {
            assert_eq!(request.atlas_slot(), *initial_slot);
        }
    }
}

#[test]
fn residency_clamps_an_untrusted_page_budget_to_the_gpu_atlas_capacity() {
    let mut state = HybridGiGlobalSdfSceneState::default();
    state.synchronize(
        Vec3::ZERO,
        &[],
        GLOBAL_SDF_MAX_RESIDENT_PAGE_COUNT.saturating_add(1),
    );

    let requests = state.dirty_page_build_requests();
    assert_eq!(requests.len(), GLOBAL_SDF_MAX_RESIDENT_PAGE_COUNT);
    assert!(
        requests
            .iter()
            .all(|request| request.atlas_slot() < GLOBAL_SDF_MAX_RESIDENT_PAGE_COUNT as u32)
    );
    assert_eq!(
        requests
            .iter()
            .map(|request| request.atlas_slot())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        GLOBAL_SDF_MAX_RESIDENT_PAGE_COUNT
    );
}
