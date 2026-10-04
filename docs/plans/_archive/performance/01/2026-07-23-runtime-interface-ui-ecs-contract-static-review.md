---
related_code:
  - zircon_runtime_interface/src/ui/ecs.rs
  - zircon_runtime_interface/src/ui/ecs
  - zircon_runtime/src/ui/surface
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/bevy/crates/bevy_ecs/src/query/filter.rs
tests:
  - zircon_runtime_interface/src/tests/ui_ecs_projection_contracts.rs
  - current-source Windows UI ECS projection tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface UI ECS projection 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/ui/ecs.rs`与`ui/ecs/**`当前源 **2/2** 个 Rust 文件、**1,229** 行已逐文件阅读，并反查surface frame与Editor debug reflector消费者。`ui_ecs_projection_contracts.rs`现有 **14** 条测试覆盖dirty-domain映射、snapshot/delta、added/removed/updated、interaction/render-only快路、legacy derived defaults、schedule/domain impacts与serde。本轮未修改源码。

## 性能结论

- `UiEcsDirtyDomains`与`UiEcsProjectionScheduleMask`为Copy布尔合同，单个domain→stage映射是常数工作；这部分不是瓶颈。
- `UiEcsProjectionSnapshot::from_nodes`对同一nodes分别执行totals、schedule mask、10-stage impacts和8-domain impacts。impact计算先收集entries，再按每个stage/domain重扫；每组建立`BTreeSet<UiNodeId>`后又转Vec，单次snapshot形成多轮O((S+D)×N logN)分组与分配。
- snapshot/delta已经携带`totals`、`schedule_mask`、`schedule_impacts`和`dirty_domain_impacts`，但同名query helpers仍从nodes/changes完整重算。查询一个stage/domain也先重建全部rows；`derived_fields_are_fresh()`同时重算四组派生值并分配临时Vec。
- `diff_from`先从previous/current各建一张BTreeMap并比较完整owned node DTO，然后再次对changes执行totals/mask/10-stage/8-domain派生。稳定或小delta若先建两份full snapshot，成本仍随全部nodes增长。以上精确补强 **PERF-MVP-278**，不重复编号。
- `node()`、`change()`等单项lookup仍线性扫描；在generation artifact拥有node-id index后应近O(1)，但必须共享同一index，不能让每个consumer私建第二张map。

## PERF-MVP-278 补充设计与验收

1. surface rebuild按tree/layout/input/render generation发布唯一immutable ECS projection artifact，nodes、node-id index和derived rows同owner；稳定generation读取只借用，stage/domain query直接读carried row或index，不重算全体。
2. changed generation以一次node遍历同时累计totals/mask和固定stage/domain buckets；canonical unique node order成立时直接append，无每bucket BTreeSet。duplicate/乱序只在发布边界验证或canonicalize一次。
3. delta由tree/component/interaction/render changed set直接生成，禁止先物化previous/current两份full projection再建双BTreeMap；changed rows共享path/metadata handle，removed row只保存必要tombstone。
4. `derived_fields_are_fresh`留给测试/诊断gate，产品普通查询不得触发；legacy payload可在deserialize后显式一次recompute并标记generation。
5. nodes 1/1k/10k/100k、changed 0/1/1%/100%、domains 1/8记录projection passes、node visits、BTree nodes/comparisons、Vec allocations、published bytes和p95：stable visits/alloc=0，单delta随changed rows，changed generation full build近O(N+output)，single-stage query无全体重算。

## 参考引擎对照

Bevy在`Changed<T>`文档中明确指出：它不是archetype filter，即使没有实体变化也会遍历全部匹配实体。Zircon因此不能仅把full projection scan改名为“changed query”；必须由surface/tree authority提供显式changed set和generation artifact，再让ECS projection消费delta。

current-source Cargo、规模复杂度counter与F4 surface/debug products trace未完成，因此该切片继续保留在 `pending.md`，不进入 `review.md`。
