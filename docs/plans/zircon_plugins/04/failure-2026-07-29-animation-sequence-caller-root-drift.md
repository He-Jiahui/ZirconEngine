---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: animation-sequence-caller-root-drift
origin_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
fixing_plan: docs/plans/zircon_plugins/04-animation.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/05
fixing_child_dir: docs/plans/zircon_plugins/04
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/sequences.rs
  - zircon_plugins/animation/runtime/src/lib.rs
  - tools/tests/test_frameworks_05_layer_direction.py
tests:
  - python -m unittest tools.tests.test_frameworks_05_layer_direction.Frameworks05LayerDirectionTests.test_animation_manager_contract_does_not_mutate_scene_world -v
  - python -m unittest tools.tests.test_frameworks_05_layer_direction -v
  - cargo +1.94.1 check -p zircon_plugin_animation_runtime --lib --locked --jobs 1 --color never
---

# Plugins04: animation sequence caller root drift

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 来源执行切片：Frameworks06 M2 的 Frameworks05 layer-direction guard hard-cut 独立复审
- 修复责任计划：`docs/plans/zircon_plugins/04-animation.md`
- 交接原因：Plugins04 已删除 `sequence` 根模块并把 Runtime helper 重新导出到插件 crate 根，但 evaluation pipeline 仍调用退役的 `crate::sequence` 路径。最低共享修复属于 Plugins04 production caller，而不是 Frameworks05 守卫。
- 生命周期键：`animation-sequence-caller-root-drift`

## 失败现象与复现证据

Frameworks05 守卫最初仍正向匹配旧调用字符串，因此完整 Python 门产生 `28/28` 假阳性。独立复审核对生产模块树后确认：

- `zircon_plugins/animation/runtime/src/lib.rs` 没有声明 `mod sequence`；
- 同一 crate root 直接 `pub use zircon_runtime::animation::apply_sequence_to_world`；
- `zircon_plugins/animation/runtime/src/evaluation/pipeline/sequences.rs:12` 仍调用 `crate::sequence::apply_sequence_to_world`。

守卫先增加新路径正向断言和旧路径负向断言后，聚焦测试精确 RED：

```text
AssertionError: 'crate::apply_sequence_to_world' not found
Ran 1 test in 20.734s
FAILED (failures=1)
```

修改前 production caller SHA-256 为 `2a50c40fc1df3c48462e1551d19ce7075acfb4a7d4cc6899affcaa03be48fe6b`。

## 最低共享层根因

Plugins04 的 scene-hook/evaluator hard cut 已删除旧 `sequence` 根模块，但一个 evaluation pipeline 调用点没有同步迁移到现有 crate-root re-export。Python 守卫只匹配旧字符串，没有验证该路径在 Rust 模块树中可解析，因而掩盖了编译边界错误。

## 架构修复验收

- production caller 直接使用 `crate::apply_sequence_to_world`；旧 `crate::sequence::apply_sequence_to_world` 在调用点和守卫契约中均被拒绝。
- Frameworks05 聚焦测试 `1/1`、完整 layer-direction `28/28` 通过。
- 通过 coordinator 在 canonical Rust 1.94.1 下运行 `zircon_plugin_animation_runtime` lib check，确认真实 Rust 模块解析成功。
- 受管编译通过后，向 Frameworks05 返回 fixed artifact，Frameworks06 才可继续其后续受管约定门。

## 禁止临时方案

- 不得恢复 `mod sequence`、重新挂载退役 `sequence.rs`、增加 alias/shim 或复制 helper。
- 不得删除 crate-root re-export、弱化守卫或仅匹配一个同名字符串来绕过 Rust 模块解析。
- 不得把受管编译前的 Python GREEN 当作该 failure 已修复。

## 修复结果与回传

- 当前实现：production caller 已从退役的 `crate::sequence::apply_sequence_to_world` 硬切到现有 crate-root re-export `crate::apply_sequence_to_world`；未恢复模块、alias、shim 或 helper 副本。修复后 caller SHA-256 为 `ff9f0cdac7e3a19a9d7220b31a3d144fd5950262ffe4ce4c62e433e8f31fd145`。
- 静态验证：同一聚焦契约先 RED（`1` failure，20.734 秒），修复后 `1/1` GREEN（17.520 秒）；完整 Frameworks05 layer-direction `28/28` GREEN（120.095 秒）；caller `rustfmt +1.94.1 --check`、guard `py_compile` 与 scoped `git diff --check` 均 exit `0`。
- 待完成：canonical Rust 1.94.1 managed `zircon_plugin_animation_runtime --lib` check 尚未运行。本 artifact 保持 `open`，不以 Python 字符串契约代替 Rust 模块解析证据。
- 回传条件：受管编译与 immutable review 通过后，使用 coordinator `failure return` 返回 Frameworks05。

## 2026-07-31 current-source static recovery

- Snapshot 1366 independent review reported C0/I0/M1. The only Minor was canonical Rust 1.94.1
  formatting in the clean crate-root re-export file; scoped rustfmt corrected that ordering without
  changing the hard-cut API. Snapshot 1367 records the post-format exact4 bytes.
- A fresh complete Frameworks05 layer-direction run first reached 27/28 because a foreign Editor01
  production-module test imported `DefaultLevelManager`. The lowest owner was returned forward to
  `docs/plans/zircon_editor/editor/01/failure-2026-07-31-authoring-world-test-concrete-level-manager.md`;
  after moving that regression to the Editor test tree, the focused guard is 1/1 GREEN and the full
  suite is 28/28 GREEN.
- The Plugins04 caller remains the crate-root `crate::apply_sequence_to_world` path, with no retired
  `sequence` module, alias, or shim. Independent post-format review of snapshot 1369 is
  C0/I0/M0; all exact4 hashes match and the ordinal fingerprint is
  `cb5985feb709ce5c0d13e8871850d752059077798f0a1e1b7fc661baaf37c32d`.
- Managed Rust module-resolution reservation `9c4084258b844b11a304f6f9df496ed8` is durably pending
  for the canonical Rust 1.94.1 lib check. Queued validation delays only fixed return; commit and
  fixed return remain pending, so this artifact stays open.

## 2026-08-05 current-source compiled-sequence reconciliation

- Production has advanced beyond the earlier crate-root raw-sequence call: the evaluation pipeline
  now imports `compile_sequence_for_world` and `apply_compiled_sequence_to_world` from the neutral
  `zircon_runtime::animation` owner and caches compiled sequences by revision. The stale guard first
  produced the required RED because it still expected `crate::apply_sequence_to_world`.
- The guard now asserts the Runtime compiled-sequence owner and both canonical compiled APIs while
  rejecting every `crate::sequence::` caller. Its exact test is `1/1` GREEN in 13.396 seconds;
  `py_compile` and scoped `git diff --check` are also exit `0`.
- The complete Frameworks05 module is not GREEN on current source: it ran 28 tests in 62.148 seconds
  and ended `27/28` because `zircon_editor/src/core/gateway/in_process.rs` contains an inline
  production-module test that imports and constructs `DefaultLevelManager`. This is the same
  Editor01 test-placement boundary described by
  `docs/plans/zircon_editor/editor/01/failure-2026-07-31-authoring-world-test-concrete-level-manager.md`,
  but the affected gateway source and canonical test file are currently attributed to the active
  `20260805-editor02-retention-page-hardcut` Session. Their ownership was not overridden.
- The declared managed Cargo check and fixed return remain pending until the support failure is
  resolved and the complete Python gate is `28/28` GREEN. This artifact therefore remains `open`.
