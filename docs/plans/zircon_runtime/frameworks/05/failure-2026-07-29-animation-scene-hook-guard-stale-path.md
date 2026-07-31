---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: animation-scene-hook-guard-stale-path
origin_plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
fixing_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/06
fixing_child_dir: docs/plans/zircon_runtime/frameworks/05
plan_link_mode: child_record_only
related_code:
  - tools/tests/test_frameworks_05_layer_direction.py
  - zircon_runtime/src/animation/sequence/apply.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/sequences.rs
tests:
  - python -m unittest tools.tests.test_frameworks_05_layer_direction.Frameworks05LayerDirectionTests.test_animation_manager_contract_does_not_mutate_scene_world -v
  - python -m unittest tools.tests.test_frameworks_05_layer_direction -v
---

# Frameworks05: Animation scene-hook guard stale path

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md`
- 来源执行切片：M2 unified G1/G2 convention gate 的 fresh Frameworks05 layer-direction 复验
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 交接原因：失败发生在 Frameworks05 自有 layer-direction guard；Plugins04 正确物理删除 Runtime fallback `animation/scene_hook` 后，守卫仍读取退役文件。最低共享修复是同步守卫到现有唯一 Runtime sequence owner，不是恢复旧模块。
- 生命周期键：`animation-scene-hook-guard-stale-path`

## 失败现象与复现证据

2026-07-29 fresh `python -m unittest tools.tests.test_frameworks_05_layer_direction -v` 运行 28 项，27 项通过，唯一错误为：

```text
FileNotFoundError: zircon_runtime/src/animation/scene_hook/sequences.rs
test_animation_manager_contract_does_not_mutate_scene_world ... ERROR
Ran 28 tests in 122.672s
FAILED (errors=1)
```

当前源码事实：

- 退役路径 `zircon_runtime/src/animation/scene_hook/sequences.rs` 已物理删除，整个 `scene_hook` Runtime fallback 正在由 Plugins04 hard cut 移除。
- Runtime concrete scene writeback helper 的当前 owner 是 `zircon_runtime/src/animation/sequence/apply.rs`，SHA-256 为 `c5f89c2a12660dabfed4d559a0c6aa031a8f8b1d59edb8127ac0846370312565`，其中直接声明 `pub fn apply_sequence_to_world`。
- Plugin production caller 修复前 SHA-256 为 `2a50c40fc1df3c48462e1551d19ce7075acfb4a7d4cc6899affcaa03be48fe6b`，当时仍直接调用 `crate::sequence::apply_sequence_to_world`。
- Frameworks05 guard 修复后 SHA-256 为 `bb84febf8e6a511c6aed7a7f98a5260173c73c68ef1351d3bbae1763e71f105d`；该文件还含其他 owner 的未提交改动，本交接不越权吸收到独立提交。

## 最低共享层根因

守卫把已经退役的 Runtime execution call site 当成永久 owner，并用 `read_text` 直接打开该文件。Plugins04 将 fallback evaluator 和 scene hook 物理删除后，真正需要保留的契约只剩两点：中立 `AnimationManager` 不得重新拥有 `scene::World`/`apply_sequence_to_world`，以及 concrete writeback helper 继续由上层 sequence owner 持有并由 plugin pipeline 直接调用。守卫路径没有随 hard cut 更新，导致约定门因文件缺失中止，未实际检查这两个不变量。

## 架构修复验收

- 将 `test_animation_manager_contract_does_not_mutate_scene_world` 的 Runtime source 读取目标硬切到 `zircon_runtime/src/animation/sequence/apply.rs`，并断言该 owner 直接声明 `pub fn apply_sequence_to_world`；不得读取或重建 `animation/scene_hook`。
- 保留对 `core/framework/animation/manager.rs` 不含 `crate::scene` 和 `apply_sequence_to_world` 的负向断言。
- 要求 Plugins04 pipeline 直接调用当前 crate-root owner `crate::apply_sequence_to_world`，并负向拒绝退役的 `crate::sequence::apply_sequence_to_world`。
- 聚焦测试 1/1、完整 Frameworks05 layer-direction 28/28 与 Plugins04 受管 Rust 编译全部通过后，Frameworks06 M2 才能重跑其后续受管 Rust 门。

## 禁止临时方案

- 不得恢复 `zircon_runtime/src/animation/scene_hook`、添加 alias/shim、生成测试占位文件或给缺失路径增加 silent fallback。
- 不得删除该测试、跳过 FileNotFoundError 或弱化中立 manager 不得触 scene world 的断言。
- 不得由本切片提交 `tools/tests/test_frameworks_05_layer_direction.py` 中其他 session 的现有 blob；修复必须进入其真实 owner 的原子 manifest 或经协调器显式联合提交。

## 修复结果与回传

- 当前状态：Frameworks05 守卫已硬切到 `zircon_runtime/src/animation/sequence/apply.rs`，同时要求当前 crate-root caller 并负向拒绝旧 `crate::sequence` 路径。Plugins04 production caller 已完成最小 hard cut。
- 当前验证：同一聚焦测试先 RED（`1` failure，20.734 秒），caller 修复后 `1/1` GREEN（17.520 秒）；完整 layer-direction `28/28` GREEN（120.095 秒）；py_compile、caller rustfmt-check 与 scoped diff-check 均 exit `0`。
- 下级交接：Plugins04 open failure 见 `docs/plans/zircon_plugins/04/failure-2026-07-29-animation-sequence-caller-root-drift.md`；静态实现已绿，canonical Rust 1.94.1 受管编译仍待完成。
- 回传条件：Plugins04 返回 fixed、Frameworks05 聚焦/完整守卫 fresh GREEN 后，再将本 artifact 经 coordinator `failure return` 移回 Frameworks06；当前不声明修复完成。
