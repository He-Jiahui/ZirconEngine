---
related_code:
  - zircon_runtime/src/core/framework/picking
  - zircon_runtime/src/tests/picking
  - zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
reference_sources:
  - dev/bevy/crates/bevy_picking/src/hover.rs
  - dev/bevy/crates/bevy_picking/src/backend.rs
  - dev/bevy/benches/benches/bevy_picking/ray_mesh_intersection.rs
tests:
  - twenty-three of twenty-three framework picking Rust files reviewed
  - six of six picking test Rust files reviewed; twenty-two tests inventoried
  - behavioral RED to GREEN for shared output resolution, blocking parity and COW isolation
  - rustfmt and scoped git diff check passed
  - current-source Cargo, allocation counters and F2/F4 product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime core framework picking逐文件性能静态审查（2026-07-18）

## 范围与覆盖

截至2026-08-02，已重新完整阅读`zircon_runtime/src/core/framework/picking/**`当前Rust文件23/23（2,069行）及`zircon_runtime/src/tests/picking/**` 6/6（889行、22个测试），并追到editor viewport adapter及既有viewport candidate generation failure。覆盖backend、ray map、hit aggregation/sort、hover map、pointer event state、primitive backend、pipeline、report和debug feed。删除易漂移的手工组合 fingerprint；动态证据未完成，因此本目录只保留在`pending.md`。

## PERF-MVP-332：每帧命中重复排序、深拷贝与临时投影

原`PickingHoverMap::from_outputs`先构造pointer `BTreeSet`，再为每个pointer全扫全部backend outputs并排序；`PickingPipelineReport`又为每个pointer分别生成sorted与hovered hits，导致同一帧同一组命中在hover和report之间最多重复三次扫描/排序和多轮`HitRecord`深clone。`hovered_hits_for_pointer`已拥有sorted vector，却仍在push时再次clone。事件派发还深clone完整previous hover map，把exit/current hits各投影到临时Vec，release后复制pressing/dragging/dragging_over三张表再clear。

本轮已做行为等价止损：backend outputs按pointer单pass分组后每pointer只排序一次；report从既有sorted slice推导blocking/hover指标并单pass聚合output counts；owned hit直接move进hover结果；previous hover用`mem::take`转移，exit/current直接借用迭代，release三张状态表以所有权转移清空；固定5-stage报告预分配容量。2026-08-02 复核又发现 pipeline 为同时交付 output 与 next-frame event state 深 clone 完整 current hover；真实 pipeline 门禁先要求两者共享 storage，再把 `PickingHoverMap` 内部收敛为 `Arc<BTreeMap<...>>`，普通帧只复制 `Arc`，`set_pointer_hits` / `remove_pointer` 通过 `Arc::make_mut` 保持 mutation 隔离。rustfmt与scoped `git diff --check`通过；focused Cargo 仍受外部 reservation 阻塞，故不把这些静态结果写成动态通过。

## 剩余架构热点与参考引擎结论

`run_picking_pipeline`仍为每次调用新建RayMap/backend output/hover/event/report/stage容器；普通帧的 `hover_map.clone()` 已是共享 storage，但取消/清理会触发 `Arc::make_mut`，尚未提供 reusable workspace 或 COW fallback 计数。`active_buttons`会为每个pointer扫描全局`button_states`并分配Vec，drag target快照也会分配临时Vec，drag-over按dragged×hovered产生事件；primitive backend是rays×primitives全乘积且没有broad phase；RayMap是cameras×pointers全乘积。debug feed仍会把固定metrics与pointer rows物化，但editor live route+debug现已从同一 `resolve_picking_outputs` 结果派生，不再二次排序。这些剩余项统一由既有PERF-MVP-332收口，不新建重复任务。

Bevy picking的`OverMap`按pointer→layer分组后只对每层depth排序，并用swap current/previous hover、clear保留容量；其ray/mesh benchmark显式覆盖10²/10⁴/10⁶ vertices。这支持下一阶段硬切`PickingPipelineWorkspace`：复用ray/group/sorted/hover/event/report buffers，以一份resolved frame同时生成hover、report和debug；当前 COW shared storage 是去除普通帧深 clone 的止损，不替代可复用双buffer与 mutation fallback 预算。scene primitive picking接Render04可见空间查询/BVH或GPU PickId，不在framework内复制renderer私有索引。

## 验收要求

按pointers 1/8/64、cameras 1/8/64、backends 1/8/32、outputs/hits 1/100/10k、primitives 1/1k/100k、drag targets×hovered 1/100/10k、debug on/off记录output scans、sorts/comparisons、HitRecord clone bytes、alloc/realloc、ray×camera probes、primitive probes、event count、CPU p50/p95/p99和RSS：每frame每hit进入resolved projection≤1、stable buffer realloc=0、previous hover全图clone=0、report/debug不二次排序；broad phase访问由query hits主导，不随total primitives无条件线性增长。priority/order/depth、blocking/non-hoverable、cancel、press/release/click/drag、multi-camera/viewports、debug feed、Editor05 candidate parity、focused Cargo及F2/F4产品trace全部通过前，本目录不得进入`review.md`。2026-07-30的current-source Cargo复试仍受协调器队列/启动恢复边界阻塞，没有测试结果；不得把此前source guard或静态复核冒充动态GREEN。
