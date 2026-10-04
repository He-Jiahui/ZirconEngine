use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::{
    AnimationConditionOperatorAsset, AnimationParameterValue, AnimationStateAsset,
    AnimationStateKindAsset, AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
    AnimationTransitionInterruptionPolicyAsset,
};
use zircon_runtime::core::resource::ResourceId;

use super::*;

#[test]
fn sampling_cache_gives_a_recent_hit_one_second_chance_at_capacity() {
    let machine = ResourceId::from_stable_label("animation.sampling.capacity");
    let machine_source = test_machine();
    let compiled = crate::compile_animation_state_machine_runtime(&machine_source).unwrap();
    let parameters = AnimationParameterSet::new();
    let projection = StateMachineParameterProjection {
        revision: parameters.revision(),
        values: &parameters,
    };
    let mut cache = StateMachineInstanceCache::default();
    for entity in 0..STATE_MACHINE_INSTANCE_CACHE_LIMIT as u64 {
        cache.state_for(
            &MachineInstanceKey::root(entity, machine),
            &compiled,
            projection,
        );
    }
    let retained = MachineInstanceKey::root(0, machine);
    cache.state_for(&retained, &compiled, projection);
    cache.state_for(
        &MachineInstanceKey::root(STATE_MACHINE_INSTANCE_CACHE_LIMIT as u64, machine),
        &compiled,
        projection,
    );

    assert_eq!(cache.entries.len(), STATE_MACHINE_INSTANCE_CACHE_LIMIT);
    assert!(cache.entries.contains_key(&retained));
    assert!(!cache
        .entries
        .contains_key(&MachineInstanceKey::root(1, machine)));
}

#[test]
fn instance_cache_reprojects_only_for_revision_or_layout_change() {
    let machine = ResourceId::from_stable_label("animation.parameters.revision");
    let first_compiled = crate::compile_animation_state_machine_runtime(&test_machine()).unwrap();
    let first_parameters =
        AnimationParameterSet::from([("speed".into(), AnimationParameterValue::Scalar(0.25))]);
    let first_projection = StateMachineParameterProjection {
        revision: first_parameters.revision(),
        values: &first_parameters,
    };
    let instance = MachineInstanceKey::root(7, machine);
    let mut cache = StateMachineInstanceCache::default();
    let initial_layout = {
        let state = cache.state_for(&instance, &first_compiled, first_projection);
        assert_eq!(
            state.parameter_values.first(),
            Some(&Some(AnimationParameterValue::Scalar(0.25)))
        );
        Arc::clone(&state.parameter_layout)
    };

    let unchanged_parameters = first_parameters.clone();
    let unchanged_projection = StateMachineParameterProjection {
        revision: unchanged_parameters.revision(),
        values: &unchanged_parameters,
    };
    let state = cache.state_for(&instance, &first_compiled, unchanged_projection);
    assert_eq!(
        state.parameter_values.first(),
        Some(&Some(AnimationParameterValue::Scalar(0.25)))
    );
    assert!(Arc::ptr_eq(&initial_layout, &state.parameter_layout));

    let mut changed_parameters = first_parameters.clone();
    changed_parameters.insert("speed".into(), AnimationParameterValue::Scalar(0.75));
    let next_revision = StateMachineParameterProjection {
        revision: changed_parameters.revision(),
        values: &changed_parameters,
    };
    let state = cache.state_for(&instance, &first_compiled, next_revision);
    assert_eq!(
        state.parameter_values.first(),
        Some(&Some(AnimationParameterValue::Scalar(0.75)))
    );

    let second_compiled = crate::compile_animation_state_machine_runtime(&test_machine()).unwrap();
    let state = cache.state_for(&instance, &second_compiled, next_revision);
    assert!(!Arc::ptr_eq(&initial_layout, &state.parameter_layout));
}

#[test]
fn instance_cache_retires_removed_entities() {
    let machine = ResourceId::from_stable_label("animation.parameters.retirement");
    let compiled = crate::compile_animation_state_machine_runtime(&test_machine()).unwrap();
    let parameters = AnimationParameterSet::new();
    let projection = StateMachineParameterProjection {
        revision: parameters.revision(),
        values: &parameters,
    };
    let mut cache = StateMachineInstanceCache::default();
    cache.state_for(&MachineInstanceKey::root(7, machine), &compiled, projection);
    cache.state_for(&MachineInstanceKey::root(8, machine), &compiled, projection);

    cache.retain_entities(&BTreeSet::from([7]));

    assert_eq!(cache.entries.len(), 1);
    assert_eq!(cache.eviction_clock.len(), 1);
    assert!(cache
        .entries
        .contains_key(&MachineInstanceKey::root(7, machine)));
    assert!(cache
        .eviction_clock
        .iter()
        .all(|instance| instance.entity() == 7));
}

#[test]
fn instance_cache_eviction_uses_a_bounded_clock_without_a_full_map_scan() {
    let source = include_str!("../state_machine_cache.rs");
    let start = source
        .find("impl StateMachineInstanceCache {")
        .expect("instance-cache implementation starts");
    let end = source[start..]
        .find("impl AnimationEvaluationPipeline {")
        .map(|offset| start + offset)
        .expect("instance-cache implementation ends");
    let implementation = &source[start..end];

    assert!(implementation.contains("eviction_clock: VecDeque"));
    assert!(implementation.contains(".pop_front()"));
    assert!(!implementation.contains(".min_by_key"));
    assert!(!implementation.contains("eviction_order"));
}

fn test_machine() -> AnimationStateMachineAsset {
    AnimationStateMachineAsset {
        name: Some("parameter cache".into()),
        entry_state: "Idle".into(),
        states: vec![AnimationStateAsset {
            name: "Idle".into(),
            kind: AnimationStateKindAsset::GraphRef {
                graph: AssetReference::from_locator(
                    AssetUri::parse("res://animation/idle.zranim").unwrap(),
                ),
            },
        }],
        transitions: vec![AnimationStateTransitionAsset {
            from_state: "Idle".into(),
            to_state: "Idle".into(),
            duration_seconds: 0.0,
            exit_time: None,
            interruption: AnimationTransitionInterruptionPolicyAsset::None,
            conditions: vec![AnimationTransitionConditionAsset {
                parameter: "speed".into(),
                operator: AnimationConditionOperatorAsset::Greater,
                value: Some(AnimationParameterValue::Scalar(0.5)),
            }],
        }],
        layers: Vec::new(),
    }
}
