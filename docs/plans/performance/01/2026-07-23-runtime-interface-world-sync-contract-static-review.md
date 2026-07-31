---
related_code:
  - zircon_runtime_interface/src/world_sync
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
reference_sources:
  - dev/bevy/crates/bevy_ecs/src/change_detection
  - dev/godot/editor/scene/scene_tree_editor.cpp
tests:
  - zircon_runtime_interface/src/tests/world_sync_contracts.rs
  - current-source Windows zircon_runtime_interface world_sync tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface world-sync 合同性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/world_sync/**`当前源 **4/4** 个 Rust 文件、**190** 行已逐文件阅读，并完整阅读对应 `world_sync_contracts.rs`。全仓 Rust 引用核对显示这些 DTO 当前仅由接口 contract tests 使用，尚未接入 runtime producer 或 editor consumer，因此本轮没有可归因的产品帧耗时，也不得把静态审查冒充 F4 动态验收。

## 性能结论

- `WorldQuery::result_for_generation`接收已经构造完成的 `Vec<EntityRow>`；matching `generation_hint`只能跳过排序，不能跳过最昂贵的 hierarchy/component 查询、JSON value 构造、String/BTreeMap 分配和行物化。该公共合同会诱导未来 producer 在得知 `NotModified` 前先完成全部工作，新增 **PERF-MVP-563**。
- stale query 对全部 rows 执行稳定 `sort_by_key`。接口没有表达 producer 已按 entity canonical order，也没有规模预算；若 runtime inspection 已输出确定顺序，gateway 再排序会重复 O(N log N) 工作。PERF-MVP-563 要求 generation 先行，并由单一 producer owner 保证或证明 canonical order。
- `InvalidationBatch.dirty/facts`是无界 owned Vec；`ComponentType`、filter 与 selector 使用 owned type-name String。它们在低频 transport control plane 可接受，但不能直接变成每帧广播真相。watch variant/root direct index、fact coalesce、entry/bytes/age budget继续归既有 **PERF-MVP-468**；共享 immutable inspection projection归 **PERF-MVP-456**。
- `EntityRow.components`采用 `BTreeMap<String, Value>`保证 wire 顺序，但会为每实体/组件建立树节点并拥有类型名与动态 JSON。F4 大层级必须分页或按选择集生成，stable generation不得重新构造整表。

## 方案与推荐

1. 保持 eager `Vec`：兼容性最好，但 `NotModified`无法成为 producer 短路，否决。
2. 增加 lazy row-builder API，旧 API 兼容委托；先判 generation，再调用一次 builder并 canonicalize rows：**推荐的低风险切片**，但按 brainstorming 规则等待用户批准后再实现。
3. 只在 runtime producer 外部预判 generation：最终仍需如此接线，但公共 API 继续允许误用，单独采用不充分。

最终架构仍由 Editor02/Runtime05 把 generation/watch 绑定到 runtime inspection producer；lazy API 只封住 DTO 层的 eager-materialization 陷阱，不新增第二套 world authority。

## 参考引擎对照

Bevy 在 ECS storage/query filter 中以 change tick/last-run 先判变化，再访问匹配组件；Godot `SceneTreeEditor`通过 tree/node 信号合并局部更新，而不是稳定帧先重建完整行再判断是否变化。Zircon 的 `generation_hint`方向正确，但必须在 row projection 前生效。

## 动态验收

1. current-source `zircon_runtime_interface` world-sync contract tests；matching/stale/none/`u64::MAX`语义与 wire tags不变。
2. lazy builder sentinel：matching generation 调用数=0；stale/none/saturated 调用数=1。
3. rows 1/1k/100k、components 0/10/100：记录producer visits、row/String/JSON/BTreeMap alloc、sort comparisons与p95；stable generation全部为0。
4. F4 hierarchy/inspector接线后，stable generation query/build/sort/clone=0；rename/reparent只处理受影响投影；invalidation entries/bytes/age有界。

动态门禁与产品接线未完成，因此该目录继续保留在 `pending.md`，不进入 `review.md`。
