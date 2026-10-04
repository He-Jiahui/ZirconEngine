use zircon_runtime::core::framework::navigation::{NavMeshAsset, NavigationGeneratedBakeSnapshot};

use super::NavigationRuntimeState;

#[test]
fn generated_mutation_epoch_rejects_equal_snapshot_after_clear_restore() {
    let mut state = NavigationRuntimeState::default();
    let snapshot = NavigationGeneratedBakeSnapshot {
        surface_entity: Some(7),
        asset: Some(NavMeshAsset::default()),
        output_asset: Some("res://navigation/generated/aba.navmesh".to_owned()),
    };

    state
        .replace_generated_snapshot(snapshot.clone())
        .expect("initial generated snapshot mutation is admitted");
    let captured = state.bake_generation_token(Some(7));
    state
        .clear_generated_snapshots()
        .expect("generated snapshot clear is admitted");
    state
        .replace_generated_snapshot(snapshot.clone())
        .expect("restored generated snapshot mutation is admitted");

    assert_eq!(state.generated_snapshot(Some(7)), snapshot);
    assert_ne!(
        state.bake_generation_token(Some(7)),
        captured,
        "clear then restore must invalidate an older bake token even when the snapshot bytes match"
    );
}

#[test]
fn generated_epoch_exhaustion_has_no_clear_side_effects() {
    let mut state = NavigationRuntimeState::default();
    state.generated_mutation_epoch = u64::MAX;
    state.generated_bakes.insert(
        Some(3),
        super::GeneratedBakeState {
            snapshot: NavigationGeneratedBakeSnapshot::empty(Some(3)),
            loaded_handle: None,
        },
    );
    let before = state.generated_snapshot(Some(3));
    assert!(state.clear_generated_snapshots().is_err());
    assert_eq!(state.generated_snapshot(Some(3)), before);
    assert_eq!(state.overlay_generation, 0);
}

#[test]
fn bake_generation_exhaustion_has_no_context_side_effects() {
    let mut state = NavigationRuntimeState::default();
    state.bake_contexts.insert(
        Some(7),
        super::BakeContextState {
            next_generation: u64::MAX,
            current_generation: u64::MAX - 1,
            last_tiled_bake: None,
        },
    );
    let before = state
        .bake_contexts
        .get(&Some(7))
        .map(|context| (context.next_generation, context.current_generation));
    assert!(state.try_advance_bake_context(Some(7)).is_err());
    let after = state
        .bake_contexts
        .get(&Some(7))
        .map(|context| (context.next_generation, context.current_generation));
    assert_eq!(after, before);
    assert!(state.bake_tasks.is_empty());
    assert!(state.dirty_bake_tasks.is_empty());
}

#[test]
fn bake_generation_max_minus_one_is_admitted_once_then_exhausted_without_mutation() {
    let mut state = NavigationRuntimeState::default();
    state.bake_contexts.insert(
        Some(11),
        super::BakeContextState {
            next_generation: u64::MAX - 1,
            current_generation: u64::MAX - 2,
            last_tiled_bake: None,
        },
    );

    let generation = state
        .try_advance_bake_context(Some(11))
        .expect("MAX-1 is the final generation that can be admitted");
    assert_eq!(generation, u64::MAX - 1);
    assert_eq!(
        state
            .bake_contexts
            .get(&Some(11))
            .map(|context| (context.current_generation, context.next_generation)),
        Some((u64::MAX - 1, u64::MAX))
    );

    let before = state
        .bake_contexts
        .get(&Some(11))
        .map(|context| (context.current_generation, context.next_generation));
    assert!(state.try_advance_bake_context(Some(11)).is_err());
    assert_eq!(
        state
            .bake_contexts
            .get(&Some(11))
            .map(|context| (context.current_generation, context.next_generation)),
        before,
        "MAX rejection must not mutate the context"
    );
}

#[test]
fn bake_generation_boundary_is_failure_atomic_for_direct_tiled_dirty_and_operation_keys() {
    for surface in [None, Some(7), Some(11), Some(13)] {
        let mut state = NavigationRuntimeState::default();
        state.bake_contexts.insert(
            surface,
            super::BakeContextState {
                next_generation: u64::MAX - 1,
                current_generation: u64::MAX - 2,
                last_tiled_bake: None,
            },
        );

        assert_eq!(
            state.try_advance_bake_context(surface).unwrap(),
            u64::MAX - 1,
            "the final non-exhausted generation is admitted for every canonical surface key"
        );
        let before_rejected_advance = state
            .bake_contexts
            .get(&surface)
            .map(|context| (context.current_generation, context.next_generation));
        assert!(state.try_advance_bake_context(surface).is_err());
        assert_eq!(
            state
                .bake_contexts
                .get(&surface)
                .map(|context| (context.current_generation, context.next_generation)),
            before_rejected_advance,
            "exhaustion rejection is side-effect free for every operation route key"
        );
    }
}
