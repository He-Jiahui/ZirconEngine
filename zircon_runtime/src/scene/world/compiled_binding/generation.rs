//! 层级绑定的失效代数与 World 内字符串驻留表。
//!
//! 根修订和目录修订用于不同粒度的缓存失效；两类驻留编号只在所属运行时表内解释。

use std::collections::HashMap;

use crate::scene::EntityId;

use super::property_path::{ComponentFieldId, PathId};

#[derive(Clone, Debug)]
// 层级根与字段路径的失效代数；重命名、重挂载和 World 替换后使已编译动画绑定可检测过期。
pub(in crate::scene::world) struct SceneBindingGenerations {
    next: u64,
    catalog_generation: u64,
    by_root: HashMap<EntityId, u64>,
    next_path_id: u64,
    path_ids: HashMap<Box<str>, PathId>,
    next_component_field_id: u64,
    component_field_ids: HashMap<Box<str>, ComponentFieldId>,
}

/// 创建空目录：修订从零开始，两类驻留编号均从一开始。
impl Default for SceneBindingGenerations {
    fn default() -> Self {
        Self {
            next: 0,
            catalog_generation: 0,
            by_root: HashMap::new(),
            next_path_id: 1,
            path_ids: HashMap::new(),
            next_component_field_id: 1,
            component_field_ids: HashMap::new(),
        }
    }
}

impl SceneBindingGenerations {
    /// 返回指定根的已发布修订；未知根为零，读取不会创建条目。
    pub(super) fn for_root(&self, root: EntityId) -> u64 {
        self.by_root.get(&root).copied().unwrap_or_default()
    }

    pub(super) const fn catalog_generation(&self) -> u64 {
        self.catalog_generation
    }

    /// 消费 `roots` 并为本次非空输入中的所有根发布同一个新修订，同时推进目录修订。
    /// 空输入保持全部计数不变；重复根只重复写同一条目。目录计数耗尽时 panic，无可恢复错误返回。
    pub(super) fn advance_roots<I>(&mut self, roots: I)
    where
        I: IntoIterator<Item = EntityId>,
    {
        let mut roots = roots.into_iter().peekable();
        if roots.peek().is_none() {
            return;
        }

        self.next = self.next.saturating_add(1);
        self.catalog_generation = self
            .catalog_generation
            .checked_add(1)
            .expect("scene binding catalog generation must not exhaust u64");
        for root in roots {
            self.by_root.insert(root, self.next);
        }
    }

    /// 替换发布时用两侧计数的上界续接修订，再写入本次提供的根。
    /// 只续接代数，不复制旧侧驻留表；空输入直接返回。计数耗尽时 panic。
    /// 参考 `World::advance_scene_binding_generations_after` 在候选发布前的调用。
    ///
    /// Invalidates replacement-world bindings past every generation published
    /// by the retired world, including when entity identifiers are reused.
    pub(super) fn advance_roots_after<I>(&mut self, previous: &Self, roots: I)
    where
        I: IntoIterator<Item = EntityId>,
    {
        let mut roots = roots.into_iter().peekable();
        if roots.peek().is_none() {
            return;
        }

        self.next = self
            .next
            .max(previous.next)
            .checked_add(1)
            .expect("scene binding generations must not exhaust u64");
        self.catalog_generation = self
            .catalog_generation
            .max(previous.catalog_generation)
            .checked_add(1)
            .expect("scene binding catalog generation must not exhaust u64");
        for root in roots {
            self.by_root.insert(root, self.next);
        }
    }

    /// 按输入文本精确复用或分配 `PathId`；新键复制为表内拥有的字符串，不保留 `path` 借用。
    /// 返回编号随所属 World 的运行时驻留表使用；计数耗尽时 panic。
    pub(super) fn intern_path(&mut self, path: &str) -> PathId {
        if let Some(id) = self.path_ids.get(path).copied() {
            return id;
        }

        let id = PathId(self.next_path_id);
        self.next_path_id = self
            .next_path_id
            .checked_add(1)
            .expect("scene path identifiers must not exhaust u64");
        self.path_ids.insert(path.into(), id);
        id
    }

    /// 按已准备的字段身份键复用或分配编号，不在此执行规范化或字段准入。
    /// 新键由表拥有；内建别名归一化和动态键保真由编译入口完成，计数耗尽时 panic。
    pub(super) fn intern_component_field(&mut self, field: &str) -> ComponentFieldId {
        if let Some(id) = self.component_field_ids.get(field).copied() {
            return id;
        }

        let id = ComponentFieldId(self.next_component_field_id);
        self.next_component_field_id = self
            .next_component_field_id
            .checked_add(1)
            .expect("scene component field identifiers must not exhaust u64");
        self.component_field_ids.insert(field.into(), id);
        id
    }
}

// Runtime-only cache generations do not participate in persistent world equality.
impl PartialEq for SceneBindingGenerations {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[cfg(test)]
#[path = "../generation/tests/alloc_free_root_advance.rs"]
mod allocation_free_root_advance_tests;
