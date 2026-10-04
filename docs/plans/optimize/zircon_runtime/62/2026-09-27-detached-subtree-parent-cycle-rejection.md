---
doc_type: milestone-detail
related_code:
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/root_normalization.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
tests:
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/raw_cycle_tests.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch/root_normalization_profile.rs
---

# Runtime62：批量摘除的现存父链环拒绝

| 里程碑 | 范围 | 状态 | 日期 | 证据 |
|---|---|---|---|---|
| Runtime1012 | prepare/remove 的局部父链校验与 covered-root normalization | `implemented_static_review_managed_validation_pending` | 2026-09-27 | [完成清单](../../../astra/features/runtime/1012-runtime62-detached-subtree-parent-cycle-rejection-completion-list.md)、[公共合同](../../../../zircon_runtime/scene/world/detached_entity_batch.md)、R 批次源快照与后续 managed 回执 |

## 确认问题与责任边界

`get_mut::<Hierarchy>` 可写 raw parent。对 `L→A→B→A` 请求 `[L]`，旧准备逻辑无法终止；请求 `[A]`、`[A,B]` 或 self-cycle root 则会因 requested-root membership 提前停止，把环内根全部过滤为空。空 preparation 随后仍能推进 commit generation，空 batch 又无法 restore。混合合法根与环内根会只删除合法部分。

本次在既有 deferred-mutation flush 后、topology rebuild 前校验所有 requested roots 的可达父链，复用 Q Runtime1010 的 `HierarchyParentChainCycle`。每批局部 visiting/completed memo 共享祖先结果，验证完成后再判断 ancestor coverage；不逐 requested child 重走完整祖先链、不创建全 World validity snapshot、不修复 raw 图。每个可达 parent row 最多读取一次，空间随可达祖先并集增长；BTreeSet membership 与既有稳定排序仍有其对数成本。空根、缺失 requested entity、generation exhausted、raw missing ancestor 的旧语义保留。

Runtime62 拥有 hierarchy 语义，Runtime08 继续拥有 generation-bound prepare/commit、move-only table/sparse/dynamic rows、ticks、observer 与生命周期发布。完整旧 `prepare_entity_subtrees` 函数作为只用于合法 forest 的性能基线保留，仅改测试局部函数名与可见性。Q 的 derived_state/hierarchy/error/topology 文件保持只读；R 使用 Q error SHA `ebbc0a939191a332329355c020066ab6fd6adbaa6e7b0d5261824cb52082fb00`，正式合批依赖以源 manifest 固定。

UE 本地参考：`SceneComponent.cpp:2366–2371` 在 attachment 变更前拒绝成环；其 `IsAttachedTo:2794–2806` 假定受控 graph，本身没有 raw-cycle 防护，不能直接复制为 Zircon 的防御方案。`MassEntityManager.cpp:928–977` 与 `MassArchetypeData.cpp:629–670` 将去重、有效集合与行删除顺序放在批量 mutation 边界。本次保留 Zircon 现有 owned-row transfer，不引入 UE ABI 或新的 graph authority。

## 行为与规模验收

- 真实 prepare/remove 覆盖尾链入环、self/2/3 环、covered cycle、混合 valid/cycle、直接 serde 回读；拒绝后存储序列化、generation/tick、lifecycle、fact、binding generation 与 topology generation 守恒，拒绝计数递增。
- 保留合法重复/覆盖根的稳定次序、先子后父摘除、table/sparse 值和 ticks；无关损坏组件不触发全 World repair。已有完整 Runtime08 payload/camera/observer 回归继续适用。
- 非 ignored 计数回归用真实 World 和组件 parent 读取，2/128 roots、2048 深链验证唯一 parent read 数；不以 source-string 检查替代行为。
- 新 Release profile：2/128 roots × separate/shared/covered，4096 深链、100k unrelated；5 warmup pairs、31 alternating pairs；计时完整旧/新 prepare API，构造、核对、drop 位于计时外。输出 parent-read 计数、raw samples 与 p50/p95/p99，不发明原计划没有的绝对时延门。
- R 的 grouped Release lane 必须包含 `runtime62_detached_parent_cycle_release_profile`、`prepared_camera_subtree_managed_scale_fixture`、`detached_entity_batch_managed_scale_fixture` 三个 ignored filters。普通非 ignored 行为由完整 lib lane 覆盖。

## 当前证据与未覆盖合同

测试先写入，再修改生产逻辑；受用户限制未运行 dynamic red、Cargo 或编译器，未提交或监控协调器。格式、局部 diff、记录结构与 inverse 证据存于 `.codex/state/session-coordinator/async-validation-batches/2026-09-27-astra-optimize-batch-r-runtime62-detached-subtree-cycle-*`；独立源码审查已通过；动态测试、性能测量与完整 G03 验收仍等待真实回执。

本片不完成整个 RSH-G03 walker 保护或 protected-component hard cut。已有 deferred flush 会先为 Hierarchy/ActiveSelf 通知遍历 subtree；若该前置 traversal 同时遇到 dirty index 与 raw cycle，仍属于独立的下层 walker 候选。有效 detached batch 摘除后，外部 parent raw 修改可能令 restore 合并图成环，这也作为后续原子 restore 候选单独记录，不能计入本片通过。普通 project serialization 仍保存现有 raw hierarchy，不借本片改变保存/加载合同。
