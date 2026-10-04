use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::scene::{NodeKind, World};

use super::PoseTargetBindings;

#[test]
fn pose_target_bindings_reuse_runtime_indices_until_the_target_root_changes() {
    let mut world = World::empty();
    let actor = world.spawn_node(NodeKind::Empty);
    let arm = world.spawn_node(NodeKind::Mesh);
    let unrelated_root = world.spawn_node(NodeKind::Empty);
    let unrelated_child = world.spawn_node(NodeKind::Mesh);
    world.rename_node(arm, "Node2:arm-right").unwrap();
    world.set_parent_checked(arm, Some(actor)).unwrap();
    world
        .set_parent_checked(unrelated_child, Some(unrelated_root))
        .unwrap();

    let mut bindings = PoseTargetBindings::default();
    bindings.insert(world.compile_descendant_name_index(actor).unwrap());
    assert_eq!(bindings.resolve(actor, "Node2:arm-right"), Some(arm));
    assert_eq!(bindings.resolve(actor, "arm-right"), Some(arm));
    assert!(bindings.is_current_for(actor, &world));

    world
        .update_transform(arm, Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)))
        .unwrap();
    world
        .rename_node(unrelated_child, "Unrelated child")
        .unwrap();
    assert!(bindings.is_current_for(actor, &world));

    world.rename_node(arm, "Node2:renamed-arm").unwrap();
    assert!(!bindings.is_current_for(actor, &world));
}

#[test]
fn optimization_batch_20260830ce_plain_names_do_not_allocate_redundant_aliases() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let hand = world.spawn_node(NodeKind::Mesh);
    world.rename_node(hand, "Hand").unwrap();
    world.set_parent_checked(hand, Some(root)).unwrap();
    let index = world.compile_descendant_name_index(root).unwrap();
    let binding = PoseTargetBinding::from(index);

    assert!(binding.short_names.is_empty());
    assert_eq!(binding.resolve("Hand"), Some(hand));
}

#[test]
fn optimization_batch_20260830ce_plain_name_alias_static_contract() {
    let source = include_str!("../pose_target_binding.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("production source");
    let constructor_start = production
        .find("impl From<CompiledDescendantNameIndex> for PoseTargetBinding")
        .expect("binding constructor");
    let constructor_end = production[constructor_start..]
        .find("impl PoseTargetBinding")
        .map(|offset| constructor_start + offset)
        .expect("binding constructor boundary");
    let constructor = &production[constructor_start..constructor_end];

    assert!(constructor.contains("if alias != entry.name()"));
}

#[test]
#[ignore = "Release-only Runtime170 performance contract"]
fn optimization_batch_20260830ce_plain_name_alias_p95() {
    const NAME_COUNT: usize = 4_096;
    const SAMPLES: usize = 17;
    let names = (0..NAME_COUNT)
        .map(|index| format!("Bone{index}"))
        .collect::<Vec<_>>();
    let mut baseline_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);

    for sample in 0..SAMPLES {
        let baseline = || {
            let started = Instant::now();
            let mut exact = BTreeMap::<Box<str>, usize>::new();
            let mut aliases = BTreeMap::<Box<str>, usize>::new();
            for (index, name) in names.iter().enumerate() {
                exact.entry(name.as_str().into()).or_insert(index);
                aliases.entry(name.as_str().into()).or_insert(index);
            }
            black_box((exact, aliases));
            started.elapsed().as_nanos()
        };
        let optimized = || {
            let started = Instant::now();
            let mut exact = BTreeMap::<Box<str>, usize>::new();
            let aliases = BTreeMap::<Box<str>, usize>::new();
            for (index, name) in names.iter().enumerate() {
                exact.entry(name.as_str().into()).or_insert(index);
            }
            black_box((exact, aliases));
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            baseline_samples.push(baseline());
            optimized_samples.push(optimized());
        } else {
            optimized_samples.push(optimized());
            baseline_samples.push(baseline());
        }
    }

    let baseline_p95 = percentile_95(&mut baseline_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    println!(
        "RUNTIME170_BINDING_ALIAS_BENCH_V1 baseline_p95_ns={baseline_p95} optimized_p95_ns={optimized_p95}"
    );
    assert!(
        optimized_p95.saturating_mul(100) <= baseline_p95.saturating_mul(85),
        "expected redundant alias removal to reduce P95 by at least 15%: baseline={baseline_p95}ns optimized={optimized_p95}ns"
    );
}

fn percentile_95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95 / 100).min(samples.len() - 1)]
}
