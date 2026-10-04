//! 缓存场景根节点的后代名称索引，供帧写回把骨骼姿态定位到实体；索引只在场景拓扑仍当前时可复用。
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
    // BUG: [CR-PLUGIN-ANIMATION-0003] 合法骨架若有重复短骨名，写回阶段仅凭 name 查首个后代，不同骨的姿态会写到同一节点；证据：pose_apply.rs 的逐骨 resolve 与目标表的完整路径身份。
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
#[path = "tests/pose_target_binding.rs"]
mod tests;
