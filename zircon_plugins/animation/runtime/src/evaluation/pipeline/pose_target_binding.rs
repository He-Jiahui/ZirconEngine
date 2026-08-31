use std::collections::BTreeMap;

use zircon_runtime::scene::world::CompiledDescendantNameIndex;
use zircon_runtime::scene::{EntityId, World};

#[derive(Debug, Default)]
pub(super) struct PoseTargetBindings {
    by_root: BTreeMap<EntityId, PoseTargetBinding>,
}

impl PoseTargetBindings {
    pub(super) fn insert(&mut self, index: CompiledDescendantNameIndex) {
        let root = index.root();
        self.by_root.insert(root, PoseTargetBinding::from(index));
    }

    pub(super) fn is_current_for(&self, root: EntityId, world: &World) -> bool {
        self.by_root
            .get(&root)
            .is_some_and(|binding| binding.index.is_current_for(world))
    }

    pub(super) fn resolve(&self, root: EntityId, bone_name: &str) -> Option<EntityId> {
        self.by_root.get(&root)?.resolve(bone_name)
    }

    pub(super) fn clear(&mut self) {
        self.by_root.clear();
    }
}

#[derive(Debug)]
struct PoseTargetBinding {
    index: CompiledDescendantNameIndex,
    exact_names: BTreeMap<Box<str>, EntityId>,
    short_names: BTreeMap<Box<str>, EntityId>,
}

impl From<CompiledDescendantNameIndex> for PoseTargetBinding {
    fn from(index: CompiledDescendantNameIndex) -> Self {
        let mut exact_names = BTreeMap::new();
        let mut short_names = BTreeMap::new();
        for entry in index.entries() {
            exact_names
                .entry(entry.name().into())
                .or_insert(entry.entity());
            let alias = short_node_name(entry.name());
            if alias != entry.name() {
                short_names.entry(alias.into()).or_insert(entry.entity());
            }
        }
        Self {
            index,
            exact_names,
            short_names,
        }
    }
}

impl PoseTargetBinding {
    fn resolve(&self, bone_name: &str) -> Option<EntityId> {
        let trimmed = bone_name.trim();
        if trimmed.is_empty() {
            return None;
        }
        let path_tail = trimmed.rsplit('/').next().unwrap_or(trimmed);
        let short_name = short_node_name(path_tail);
        let candidates = [trimmed, path_tail, short_name];

        for index in 0..candidates.len() {
            let candidate = candidates[index];
            if candidates[..index].contains(&candidate) {
                continue;
            }
            if let Some(entity) = self.exact_names.get(candidate).copied() {
                return Some(entity);
            }
        }
        for index in 0..candidates.len() {
            let candidate = candidates[index];
            if candidates[..index].contains(&candidate) {
                continue;
            }
            if let Some(entity) = self.short_names.get(candidate).copied() {
                return Some(entity);
            }
        }
        None
    }
}

fn short_node_name(name: &str) -> &str {
    name.rsplit_once(':')
        .map(|(_, short)| short.trim())
        .unwrap_or(name.trim())
}

#[cfg(test)]
mod tests {
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
        let source = include_str!("pose_target_binding.rs");
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
}
