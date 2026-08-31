---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-22
summary_slug: world-deserialize-node-cache-initializer
origin_plan: docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
fixing_plan: docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
origin_child_dir: docs/plans/zircon_runtime/runtime/07
fixing_child_dir: docs/plans/zircon_runtime/runtime/07
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/world/world.rs:454; docs/plans/zircon_runtime/runtime/07/2026-08-22-m2-world-derived-state-generation-topology-manifest.md
---

# world-deserialize-node-cache-initializer: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md`
- 来源执行切片：Render11 Shader06 realtime IBL managed library validation
- 修复责任计划：`docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Render11 Shader06 realtime IBL managed library validation` — Windows managed validate-matrix compilation of zircon_runtime with text_oversized_run_keeps_one_logical_shaped_line reaches zircon_runtime/src/scene/world/world.rs:454 and fails E0063 because World::from_persistent_state omits node_cache_rows and node_cache_topology_generation.

## 最低共享层根因

Runtime07 F459 M2 added World node-cache projection state and Clone propagation but did not initialize the two fields in the serialized-world Self initializer.

## 架构修复验收

- The serialized-world constructor initializes both fields consistently with the canonical bootstrap/default path, and the originating managed zircon_runtime validation advances past world.rs E0063 without a compatibility shim.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

Open state: `实现已修复，等待上行验证`.

- 已在 `World::from_persistent_state` 初始化 `node_cache_rows` 与
  `node_cache_topology_generation`，并在派生状态回归模块加入结构约束与
  `deserialize -> first flush -> checked reparent` 行为回归。
- Windows managed `zircon_runtime` production build 已在非 C 盘目标池
  `D:\\cargo-targets\\zircon-engine\\pool\\f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d`
  成功完成：Cargo job `a97a3972585e4baf9736dad06c990105`，耗时 13m42s。该结果确认
  `world.rs:454` 的 E0063 已消失，但不执行 lib-test 行为断言。
- 原始上行 focused lib-test 已以 Cargo job `d8540e5eed3d4f38b1c5010b3993937f` 提交，且已越过
  此前的 `world.rs:454` 缺失字段诊断；在执行目标测试前，整个测试 harness 因 19 个 Runtime74
  UI 契约错误失败：15 个 `UiAssetLoader::load_str` 调用、两处未标注类型的
  `serialized.try_into()`、binding-ownership 计数作用域和一处
  `UiBindingMutationTransaction::commit()` 旧签名。关联的 Runtime74 开放失败记录为
  `docs/plans/optimize/zircon_runtime/74/failure-2026-08-22-ui-asset-binding-canonical-loader-api-tests.md`
  与 `docs/plans/optimize/zircon_runtime/74/failure-2026-08-22-text03-compiled-binding-contract-compile.md`。
- 该记录保持 open；只有原始上行验证通过后才可由协调器返回为 `fixed-*`。
