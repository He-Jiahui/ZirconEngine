//! World 的运行时修订：整体同步、生命周期可见性和逐动态组件载荷分别计数。
//!
//! 这些修订不等同于模式目录代数；反序列化重建与替换发布也使用不同的续接入口。

use super::World;

/// Runtime-only monotonic revision for observable world mutations.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct WorldGeneration(u64);

/// Prepared while the candidate World is exclusively owned. The borrowed
/// mutation recorder cannot coexist with that exclusive ownership, and cloning
/// a sink copies its records into a new sink rather than sharing an Arc.
#[derive(Clone, Copy, Debug)]
pub(in crate::scene) struct PreparedWorldGenerationAfter {
    base: u64,
    effective: u64,
}

impl PreparedWorldGenerationAfter {
    pub(in crate::scene) const fn generation(self) -> u64 {
        self.effective
    }
}

impl WorldGeneration {
    pub(super) const fn get(self) -> u64 {
        self.0
    }

    fn checked_after(self, previous: Self, pending: u64) -> Option<PreparedWorldGenerationAfter> {
        let base = self.0.max(previous.0).checked_add(1)?;
        let effective = base.checked_add(pending)?;
        Some(PreparedWorldGenerationAfter { base, effective })
    }

    fn advance(&mut self) {
        self.0 = self.0.saturating_add(1);
    }

    /// 返回加上 `count` 后的饱和修订副本；克隆 World 用它合入尚未结算的组件变更。
    pub(super) const fn advanced_by(self, count: u64) -> Self {
        Self(self.0.saturating_add(count))
    }

    /// Carries a session revision across wholesale world replacement.
    fn advance_after(&mut self, previous: Self) {
        self.0 = self.0.max(previous.0).saturating_add(1);
    }
}

// Runtime revisions do not participate in persistent world equality.
impl PartialEq for WorldGeneration {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

/// Runtime-only revision published with component lifecycle visibility.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct LifecycleVisibilityRevision(u64);

impl LifecycleVisibilityRevision {
    pub(super) const fn get(self) -> u64 {
        self.0
    }

    pub(super) fn advance(&mut self) {
        self.0 = self.0.saturating_add(1);
    }
}

// Runtime revisions do not participate in persistent world equality.
impl PartialEq for LifecycleVisibilityRevision {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl World {
    /// 读取用于同步与缓存失效的 World 运行时修订。
    ///
    /// # 返回
    /// 已记账修订与待结算组件可变访问计数的饱和和；成功借出可变组件也会记为变更。
    /// 此值不代表值比较结果或持久化版本。持久化重建从零开始，替换发布通过
    /// `advance_world_generation_after` 续接旧值；组件借出语义参见 [`Self::get_mut`]。
    ///
    /// Returns the current runtime synchronization revision.
    pub fn world_generation(&self) -> u64 {
        self.world_generation
            .get()
            .saturating_add(self.derived_state_dirty.pending_component_mutation_count())
    }

    /// Checks both the stored revision and its actual pending mutation count.
    /// The candidate must remain exclusively owned until commit; neither this
    /// preparation nor commit drains records or invokes a lifecycle callback.
    pub(in crate::scene) fn prepare_world_generation_after(
        &self,
        previous: u64,
    ) -> Option<PreparedWorldGenerationAfter> {
        self.world_generation.checked_after(
            WorldGeneration(previous),
            self.derived_state_dirty.pending_component_mutation_count(),
        )
    }

    /// Installs the exact checked base in the same exclusively owned candidate.
    /// No mutation recorder may be created between preparation and this call.
    pub(in crate::scene) fn commit_prepared_world_generation(
        &mut self,
        prepared: PreparedWorldGenerationAfter,
    ) {
        self.world_generation = WorldGeneration(prepared.base);
    }

    /// 读取组件生命周期可见性修订；它与一般 World 修订分别推进。
    /// 组件行重建会先推进此值再标脏，回调所见时序由组件发布入口决定。
    ///
    /// Returns the revision observed by component lifecycle callbacks.
    pub(crate) fn lifecycle_visibility_revision(&self) -> u64 {
        self.lifecycle_visibility_revision.get()
    }

    /// 按 `component_id` 精确键读取动态载荷修订；尚无记录时返回零。
    /// 载荷增删改推进该值，模式定义的变化则由 [`Self::component_type_schema_generation`] 表示。
    ///
    /// Returns the revision for one dynamic component type only.
    ///
    /// Consumers that project a single component can avoid rebuilding after
    /// unrelated world mutations such as transform updates.
    pub fn dynamic_component_generation(&self, component_id: &str) -> u64 {
        self.dynamic_component_generations
            .get(component_id)
            .copied()
            .unwrap_or_default()
    }

    pub(super) fn advance_world_generation(&mut self) {
        // The ordinary entry owns the generation field exclusively.
        unsafe { Self::advance_world_generation_unchecked(self) }
    }

    /// 只写入原始指针所指 World 的修订叶子，不创建覆盖整个 World 的可变引用。
    /// `world` 须指向存活且正确对齐的 World；计数字段必须可独占写入，存活借用不能覆盖它或其父对象。
    /// 查询行或变更记录器仅可保留与该字段分离的叶子；调用准入见 `mark_component_mutation_unchecked`。
    ///
    /// # Safety
    /// The caller must hold exclusive access to the generation leaf. Live query
    /// items may retain row/sink leaves, but must not retain this field or its parents.
    pub(super) unsafe fn advance_world_generation_unchecked(world: *mut Self) {
        unsafe {
            (&mut *std::ptr::addr_of_mut!((*world).world_generation)).advance();
        }
    }

    /// 推进组件生命周期可见性的独立饱和修订；不改动一般 World 修订。
    pub(super) fn bump_lifecycle_visibility_revision(&mut self) {
        self.lifecycle_visibility_revision.advance();
    }

    /// 为精确组件键创建或推进一条饱和载荷修订；不修改该类型的模式修订。
    pub(super) fn advance_dynamic_component_generation(&mut self, component_id: &str) {
        let generation = self
            .dynamic_component_generations
            .entry(component_id.to_string())
            .or_default();
        *generation = generation.saturating_add(1);
    }

    /// 在替换发布前续接旧 World 与候选 World 的动态载荷修订。
    /// `previous` 只借用至本次调用；两侧计数键与实际载荷键的并集逐键取较大值再饱和加一，
    /// 因此旧侧已移除、或新侧刚出现的载荷也参与失效。调用顺序见 `commit_staged_scene_state`
    /// 与 `LevelSystem::replace_world_and_reset_runtime_state`。
    ///
    /// Carries component-specific projection revisions across whole-world replacement.
    ///
    /// A deserialized world intentionally starts with no runtime revisions. Replacement must still
    /// invalidate a cached projection for both removed and newly introduced component types.
    pub(in crate::scene) fn advance_dynamic_component_generations_after(
        &mut self,
        previous: &Self,
    ) {
        let mut component_ids = previous
            .dynamic_component_generations
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        component_ids.extend(
            previous
                .dynamic_components
                .values()
                .flat_map(|components| components.keys().cloned()),
        );
        component_ids.extend(self.dynamic_component_generations.keys().cloned());
        component_ids.extend(
            self.dynamic_components
                .values()
                .flat_map(|components| components.keys().cloned()),
        );
        component_ids.sort_unstable();
        component_ids.dedup();

        for component_id in component_ids {
            let prior = previous.dynamic_component_generation(&component_id);
            let staged = self.dynamic_component_generation(&component_id);
            self.dynamic_component_generations
                .insert(component_id, prior.max(staged).saturating_add(1));
        }
    }

    /// 将 `self` 的已记账计数与 `previous` 取较大值后饱和加一。
    /// 替换调用方传入旧 World 的公开修订，以包含其待结算变更；本函数不消费或清空待结算记录。
    pub(in crate::scene) fn advance_world_generation_after(&mut self, previous: u64) {
        self.world_generation
            .advance_after(WorldGeneration(previous));
    }
}

#[cfg(test)]
#[path = "generation/tests/cases.rs"]
mod tests;

#[cfg(test)]
mod replacement_generation_tests {
    use super::*;

    #[test]
    fn checked_replacement_revision_counts_pending_once_and_rejects_exhaustion() {
        // The production owner helper receives the sink's actual count. These
        // boundary inputs test that helper, not a second copy of its arithmetic.
        let prepared = WorldGeneration(u64::MAX - 2)
            .checked_after(WorldGeneration(0), 1)
            .expect("the final effective revision is still representable");
        assert_eq!(prepared.base, u64::MAX - 1);
        assert_eq!(prepared.generation(), u64::MAX);
        assert!(WorldGeneration(u64::MAX - 2)
            .checked_after(WorldGeneration(0), 2)
            .is_none());
        assert!(WorldGeneration(u64::MAX)
            .checked_after(WorldGeneration(0), 0)
            .is_none());
        assert!(WorldGeneration(0)
            .checked_after(WorldGeneration(u64::MAX), 0)
            .is_none());

        let mut world = World::empty();
        let before = world.world_generation();
        let prepared = world
            .prepare_world_generation_after(before)
            .expect("a real newly owned World can advance");
        let effective = prepared.generation();
        assert_eq!(world.world_generation(), before);
        world.commit_prepared_world_generation(prepared);
        assert_eq!(world.world_generation(), effective);
        assert!(effective > before);
    }
}
