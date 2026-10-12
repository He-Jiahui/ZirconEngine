use super::*;

#[test]
fn hybrid_gi_scene_representation_seeds_radiance_cache_from_surface_cache_then_voxel_fallback() {
    let mut representation =
        HybridGiSceneRepresentation::from_extract(&extract_with_trace_and_budgets(2, 2, 1));
    let meshes = [
        mesh_at(11, Vec3::new(-1.0, 0.0, 0.0), 2.0),
        mesh_at(22, Vec3::new(3.0, 0.0, 0.0), 1.0),
    ];

    representation.synchronize_scene(&meshes, &[], &[], &[]);
    representation
        .surface_cache_mut()
        .replace_page_contents_for_test(&[(0, 11, 0, 0, [10, 20, 30, 255], [40, 50, 60, 255])]);
    representation.synchronize_scene(&meshes, &[], &[], &[]);

    let entries = representation.radiance_cache_entries();
    assert_eq!(
        entries[0],
        (0, 11, Some(0), [40, 50, 60], 255, "surface-cache")
    );
    assert_eq!(entries[1].0, 1);
    assert_eq!(entries[1].1, 22);
    assert_eq!(entries[1].2, Some(1));
    assert_ne!(entries[1].3, [0, 0, 0]);
    assert_eq!(entries[1].4, 128);
    assert_eq!(entries[1].5, "voxel-fallback");
}

#[test]
fn hybrid_gi_radiance_cache_commits_surface_radiance_into_resident_lattice_slots() {
    let mut representation =
        HybridGiSceneRepresentation::from_extract(&extract_with_trace_and_budgets(1, 1, 0));
    let meshes = [mesh_at(11, Vec3::ZERO, 1.0)];

    representation.synchronize_scene(&meshes, &[], &[], &[]);
    representation
        .surface_cache_mut()
        .replace_page_contents_for_test(&[(0, 11, 0, 0, [10, 20, 30, 255], [40, 50, 60, 255])]);
    representation.synchronize_scene(&meshes, &[], &[], &[]);

    let resident_samples = representation.radiance_cache_resident_samples();
    assert_eq!(resident_samples.len(), 8);
    assert!(resident_samples
        .iter()
        .all(|(_, _, radiance, confidence, source)| {
            *radiance == [40, 50, 60] && *confidence == 255 && *source == "surface-cache"
        }));
}

#[test]
fn hybrid_gi_radiance_cache_marks_unique_probe_lattice_demands() {
    let mut representation =
        HybridGiSceneRepresentation::from_extract(&extract_with_trace_and_budgets(2, 2, 0));

    representation.synchronize_scene(
        &[mesh_at(11, Vec3::ZERO, 1.0), mesh_at(22, Vec3::ZERO, 1.0)],
        &[],
        &[],
        &[],
    );

    assert_eq!(representation.screen_probe_count(), 2);
    assert_eq!(
        representation.radiance_cache_probe_demands(),
        vec![
            (0, [23, 23, 23]),
            (0, [23, 23, 24]),
            (0, [23, 24, 23]),
            (0, [23, 24, 24]),
            (0, [24, 23, 23]),
            (0, [24, 23, 24]),
            (0, [24, 24, 23]),
            (0, [24, 24, 24]),
        ]
    );
}

#[test]
fn hybrid_gi_radiance_cache_clipmap_topology_is_independent_from_voxel_budget() {
    let meshes = [
        mesh_at(11, Vec3::ZERO, 1.0),
        mesh_at(22, Vec3::new(40.0, 0.0, 0.0), 1.0),
        mesh_at(33, Vec3::new(80.0, 0.0, 0.0), 1.0),
        mesh_at(44, Vec3::new(160.0, 0.0, 0.0), 1.0),
    ];
    let mut without_voxel =
        HybridGiSceneRepresentation::from_extract(&extract_with_trace_and_budgets(4, 4, 0));
    let mut with_voxel =
        HybridGiSceneRepresentation::from_extract(&extract_with_trace_and_budgets(4, 4, 4));

    without_voxel.synchronize_scene(&meshes, &[], &[], &[]);
    with_voxel.synchronize_scene(&meshes, &[], &[], &[]);

    assert_eq!(without_voxel.voxel_scene().resident_clipmap_count(), 0);
    assert_eq!(with_voxel.voxel_scene().resident_clipmap_count(), 4);
    assert_eq!(
        without_voxel.radiance_cache_clipmap_topology(),
        vec![(0, 48, 1.0), (1, 48, 2.0), (2, 48, 4.0), (3, 48, 8.0)]
    );
    assert_eq!(
        without_voxel.radiance_cache_clipmap_topology(),
        with_voxel.radiance_cache_clipmap_topology()
    );
    assert_eq!(
        without_voxel.radiance_cache_probe_demands(),
        with_voxel.radiance_cache_probe_demands()
    );
    let demands = without_voxel.radiance_cache_probe_demands();
    assert_eq!(demands.len(), 32);
    assert_eq!(demands.iter().filter(|(level, _)| *level == 0).count(), 8);
    assert_eq!(demands.iter().filter(|(level, _)| *level == 1).count(), 8);
    assert_eq!(demands.iter().filter(|(level, _)| *level == 2).count(), 8);
    assert_eq!(demands.iter().filter(|(level, _)| *level == 3).count(), 8);
    assert!(demands.contains(&(1, [43, 23, 23])));
    assert!(demands.contains(&(1, [44, 24, 24])));
    assert!(demands.contains(&(2, [43, 23, 23])));
    assert!(demands.contains(&(2, [44, 24, 24])));
    assert!(demands.contains(&(3, [43, 23, 23])));
    assert!(demands.contains(&(3, [44, 24, 24])));
}

#[test]
fn hybrid_gi_radiance_cache_selects_clipmaps_symmetrically_at_strict_edges() {
    let mut representation =
        HybridGiSceneRepresentation::from_extract(&extract_with_trace_and_budgets(3, 3, 0));

    representation.synchronize_scene(
        &[
            mesh_at(11, Vec3::ZERO, 1.0),
            mesh_at(22, Vec3::new(-23.5, 0.0, 0.0), 1.0),
            mesh_at(33, Vec3::new(23.5, 0.0, 0.0), 1.0),
        ],
        &[],
        &[],
        &[],
    );

    let demands = representation.radiance_cache_probe_demands();
    assert_eq!(demands.len(), 24);
    assert_eq!(demands.iter().filter(|(level, _)| *level == 0).count(), 8);
    assert_eq!(demands.iter().filter(|(level, _)| *level == 1).count(), 16);
    assert!(demands.contains(&(1, [11, 23, 23])));
    assert!(demands.contains(&(1, [12, 24, 24])));
    assert!(demands.contains(&(1, [35, 23, 23])));
    assert!(demands.contains(&(1, [36, 24, 24])));
}

#[test]
fn hybrid_gi_radiance_cache_rejects_invalid_positions_and_clears_empty_scene() {
    let mut representation =
        HybridGiSceneRepresentation::from_extract(&extract_with_trace_and_budgets(3, 3, 0));

    representation.synchronize_scene(
        &[
            mesh_at(11, Vec3::splat(f32::NAN), 1.0),
            mesh_at(22, Vec3::new(8.0, -4.0, 2.0), 1.0),
            mesh_at(33, Vec3::splat(f32::MAX), 1.0),
        ],
        &[],
        &[],
        &[],
    );

    assert_eq!(representation.screen_probe_count(), 3);
    assert_eq!(
        representation.radiance_cache_clipmap_topology(),
        vec![(0, 48, 1.0), (1, 48, 2.0), (2, 48, 4.0), (3, 48, 8.0)]
    );
    assert_eq!(
        representation.radiance_cache_probe_demands(),
        vec![
            (0, [23, 23, 23]),
            (0, [23, 23, 24]),
            (0, [23, 24, 23]),
            (0, [23, 24, 24]),
            (0, [24, 23, 23]),
            (0, [24, 23, 24]),
            (0, [24, 24, 23]),
            (0, [24, 24, 24]),
        ]
    );
    assert_eq!(representation.radiance_cache_entry_count(), 3);

    representation.synchronize_scene(&[], &[], &[], &[]);

    assert_eq!(representation.radiance_cache_entry_count(), 0);
    assert!(representation.radiance_cache_clipmap_topology().is_empty());
    assert!(representation.radiance_cache_probe_demands().is_empty());
}

#[test]
fn hybrid_gi_radiance_cache_retains_slots_and_generation_for_unchanged_demands() {
    let mut representation =
        HybridGiSceneRepresentation::from_extract(&extract_with_trace_and_budgets(2, 2, 0));
    let meshes = [
        mesh_at(11, Vec3::ZERO, 1.0),
        mesh_at(22, Vec3::new(40.0, 0.0, 0.0), 1.0),
    ];

    representation.synchronize_scene(&meshes, &[], &[], &[]);
    representation.synchronize_scene(&meshes, &[], &[], &[]);
    let first_residents = representation.radiance_cache_resident_probes();

    representation.synchronize_scene(&meshes, &[], &[], &[]);
    let second_residents = representation.radiance_cache_resident_probes();

    assert_eq!(first_residents.len(), 16);
    assert_eq!(
        first_residents
            .iter()
            .map(|(level, coord, slot, generation, _, _, epoch)| {
                (*level, *coord, *slot, *generation, *epoch)
            })
            .collect::<Vec<_>>(),
        second_residents
            .iter()
            .map(|(level, coord, slot, generation, _, _, epoch)| {
                (*level, *coord, *slot, *generation, *epoch)
            })
            .collect::<Vec<_>>()
    );
    assert!(first_residents
        .iter()
        .zip(&second_residents)
        .all(|(first, second)| first.4 < second.4 && first.5 == second.5));
}

#[test]
fn hybrid_gi_radiance_cache_retraces_retained_slots_when_participation_epoch_advances() {
    let mut representation =
        HybridGiSceneRepresentation::from_extract(&extract_with_trace_and_budgets(1, 1, 0));
    let original_mesh = mesh_at(11, Vec3::ZERO, 1.0);

    representation.synchronize_scene(&[original_mesh.clone()], &[], &[], &[]);
    representation.synchronize_scene(&[original_mesh], &[], &[], &[]);
    let before_change = representation.radiance_cache_resident_probes();

    representation.synchronize_scene(
        &[mesh_at(11, Vec3::new(0.25, 0.0, 0.0), 1.0)],
        &[],
        &[],
        &[],
    );
    let after_change = representation.radiance_cache_resident_probes();

    assert_eq!(before_change.len(), 8);
    assert_eq!(
        before_change
            .iter()
            .map(|(level, coord, slot, _, _, _, _)| (*level, *coord, *slot))
            .collect::<Vec<_>>(),
        after_change
            .iter()
            .map(|(level, coord, slot, _, _, _, _)| (*level, *coord, *slot))
            .collect::<Vec<_>>()
    );
    assert!(before_change
        .iter()
        .zip(&after_change)
        .all(|(before, after)| before.3 < after.3 && before.5 < after.5 && before.6 < after.6));
}

#[test]
fn hybrid_gi_radiance_cache_truncates_over_budget_demands_to_deterministic_missing_fallbacks() {
    let mut representation =
        HybridGiSceneRepresentation::from_extract(&extract_with_trace_and_budgets(5, 5, 0));
    let meshes = [
        mesh_at(11, Vec3::ZERO, 1.0),
        mesh_at(22, Vec3::new(40.0, 0.0, 0.0), 1.0),
        mesh_at(33, Vec3::new(80.0, 0.0, 0.0), 1.0),
        mesh_at(44, Vec3::new(120.0, 0.0, 0.0), 1.0),
        mesh_at(55, Vec3::new(160.0, 0.0, 0.0), 1.0),
    ];

    representation.synchronize_scene(&meshes, &[], &[], &[]);
    assert_eq!(
        representation.radiance_cache_last_sampled_demand_count(),
        32
    );
    representation.synchronize_scene(&meshes, &[], &[], &[]);
    assert_eq!(representation.radiance_cache_last_sampled_demand_count(), 0);
    let first_residents = representation.radiance_cache_resident_probes();
    let first_entries = representation.radiance_cache_entries();

    representation.synchronize_scene(&meshes, &[], &[], &[]);
    let second_residents = representation.radiance_cache_resident_probes();

    assert_eq!(representation.radiance_cache_probe_demands().len(), 40);
    assert_eq!(first_residents.len(), 32);
    assert!(first_entries
        .iter()
        .any(|entry| { entry.3 == [0, 0, 0] && entry.4 == 0 && entry.5 == "missing" }));
    assert_eq!(
        first_residents
            .iter()
            .map(|(level, coord, slot, generation, _, _, epoch)| {
                (*level, *coord, *slot, *generation, *epoch)
            })
            .collect::<Vec<_>>(),
        second_residents
            .iter()
            .map(|(level, coord, slot, generation, _, _, epoch)| {
                (*level, *coord, *slot, *generation, *epoch)
            })
            .collect::<Vec<_>>()
    );
}

#[test]
fn hybrid_gi_radiance_cache_camera_scroll_propagates_overlap_without_retracing() {
    let mut representation =
        HybridGiSceneRepresentation::from_extract(&extract_with_trace_and_budgets(1, 1, 0));
    let meshes = [mesh_at(11, Vec3::ZERO, 1.0)];

    representation.synchronize_scene_with_baked_and_view_state(
        &meshes,
        &[],
        &[],
        &[],
        None,
        false,
        Some(Vec3::ZERO),
        false,
    );
    representation.synchronize_scene_with_baked_and_view_state(
        &meshes,
        &[],
        &[],
        &[],
        None,
        false,
        Some(Vec3::ZERO),
        false,
    );
    let before_scroll = representation.radiance_cache_resident_probes();

    representation.synchronize_scene_with_baked_and_view_state(
        &meshes,
        &[],
        &[],
        &[],
        None,
        false,
        Some(Vec3::new(0.5, 0.0, 0.0)),
        false,
    );
    let after_subcell_motion = representation.radiance_cache_resident_probes();

    assert_eq!(representation.radiance_cache_scroll_count(), 0);
    assert_eq!(
        before_scroll
            .iter()
            .map(|(level, coord, slot, generation, _, _, epoch)| {
                (*level, *coord, *slot, *generation, *epoch)
            })
            .collect::<Vec<_>>(),
        after_subcell_motion
            .iter()
            .map(|(level, coord, slot, generation, _, _, epoch)| {
                (*level, *coord, *slot, *generation, *epoch)
            })
            .collect::<Vec<_>>()
    );
    assert!(before_scroll
        .iter()
        .zip(&after_subcell_motion)
        .all(|(before, after)| before.4 < after.4 && before.5 == after.5));

    representation.synchronize_scene_with_baked_and_view_state(
        &meshes,
        &[],
        &[],
        &[],
        None,
        false,
        Some(Vec3::new(1.0, 0.0, 0.0)),
        false,
    );
    let after_scroll = representation.radiance_cache_resident_probes();
    let gpu_updates = representation.radiance_cache_gpu_updates();

    assert_eq!(representation.radiance_cache_scroll_count(), 1);
    assert_eq!(before_scroll.len(), 8);
    assert_eq!(after_scroll.len(), 8);
    assert_ne!(
        before_scroll
            .iter()
            .map(|(level, coord, _, _, _, _, _)| (*level, *coord))
            .collect::<Vec<_>>(),
        after_scroll
            .iter()
            .map(|(level, coord, _, _, _, _, _)| (*level, *coord))
            .collect::<Vec<_>>()
    );
    for before in &before_scroll {
        let propagated_coord = [before.1[0] - 1, before.1[1], before.1[2]];
        let after = after_scroll
            .iter()
            .find(|after| after.0 == before.0 && after.1 == propagated_coord)
            .expect("a one-cell camera scroll must retain each overlapping world-space probe");
        assert_eq!(after.2, before.2, "scroll propagation must retain the slot");
        assert!(after.3 > before.3, "the visible generation must advance");
        assert!(after.4 > before.4, "the retained probe must be marked used");
        assert_eq!(
            after.5, before.5,
            "an overlapping probe must not be retraced during scroll propagation"
        );
    }
    assert_eq!(representation.radiance_cache_last_sampled_demand_count(), 0);
    assert_eq!(representation.radiance_cache_update_probe_count(), 0);
    assert_eq!(gpu_updates.len(), after_scroll.len());
    assert!(gpu_updates
        .iter()
        .all(|update| update.reuse_committed_radiance));
}

#[test]
fn hybrid_gi_radiance_cache_history_invalidation_discards_prior_generation() {
    let mut representation =
        HybridGiSceneRepresentation::from_extract(&extract_with_trace_and_budgets(1, 1, 0));
    let meshes = [mesh_at(11, Vec3::ZERO, 1.0)];

    representation.synchronize_scene_with_baked_and_view_state(
        &meshes,
        &[],
        &[],
        &[],
        None,
        false,
        Some(Vec3::ZERO),
        false,
    );
    representation.synchronize_scene_with_baked_and_view_state(
        &meshes,
        &[],
        &[],
        &[],
        None,
        false,
        Some(Vec3::ZERO),
        false,
    );
    let before_clear = representation.radiance_cache_resident_probes();

    representation.synchronize_scene_with_baked_and_view_state(
        &meshes,
        &[],
        &[],
        &[],
        None,
        false,
        Some(Vec3::ZERO),
        true,
    );
    let after_clear = representation.radiance_cache_resident_probes();

    assert_eq!(representation.radiance_cache_history_clear_count(), 1);
    assert_eq!(before_clear.len(), 8);
    assert_eq!(after_clear.len(), 8);
    assert!(before_clear
        .iter()
        .zip(&after_clear)
        .all(|(before, after)| before.3 < after.3 && before.5 < after.5));
}
