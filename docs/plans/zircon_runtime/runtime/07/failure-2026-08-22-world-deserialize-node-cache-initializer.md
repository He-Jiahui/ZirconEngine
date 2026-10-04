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
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/tests/derived_state/hierarchy_rebuild.rs
  - zircon_runtime/src/scene/tests/derived_state/hierarchy_behavior.rs
  - zircon_runtime/src/scene/tests/derived_state.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/07/2026-08-22-m2-world-derived-state-generation-topology-manifest.md
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter deserialized_world_rebuilds_node_cache_before_incremental_reparent_projection -VerboseOutput
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter derived_state_structured_reparent_avoids_global_hierarchy_work_at_one_hundred_thousand_nodes -VerboseOutput
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter text_oversized_run_keeps_one_logical_shaped_line -VerboseOutput
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

### 2026-09-24 handoff 路径与待验门禁更正

- 原 `related_code` 把历史 `world.rs:454` 行号与一份计划 manifest 用分号拼成单个非文件路径；该原始编译位置仍保留在上方复现证据。现拆为实际存在的生产/回归文件，计划 manifest 移至 `plan_sources`，并按原受管 job `d8540e5eed3d4f38b1c5010b3993937f` 的 focused 过滤器和原始 Render11/Shader06 文本回归列出待验命令，未重复提交请求。
- 当前 `World::from_persistent_state` 源码静态可见 `node_cache_rows: HashMap::new()` 与 `node_cache_topology_generation: 0`；但 `world.rs` 已有其他会话修改（当前 SHA-256 `45d8bcaf11d5287f860c0f6be23aa3507b21f9b3d13d1a0ed56ffc2922d540e1` 与旧 attribution 不一致）。本项 session 只接管并更正 failure 文档，不修改、不声明拥有该源码，也不复用历史编译/测试为当前快照通过。
- 100,000-node 下层回归、原始 lib-test 与上行门禁需要由原源码 owner 的匹配快照受管验收；在此之前继续 `open / validation_pending`，不得回传或 closeout。

### 2026-09-24 独立审查回执

- 对文档专属快照 `3748` 的独立审查结果：Critical 0 / Important 0 / Moderate 0。`related_code` 中四条路径及 `plan_sources` 均存在；三个 `TestFilter` 分别精确匹配实际测试，受管命令参数有效。
- 审查不等于动态验收：当前 `world.rs` 不属本 session 的源码快照；待其源码 owner 完成匹配快照的下层、原始及上行受管回归后，方可回传并关闭本项。
