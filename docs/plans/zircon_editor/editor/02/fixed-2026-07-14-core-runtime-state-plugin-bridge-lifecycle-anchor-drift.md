---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: core-runtime-state-plugin-bridge-lifecycle-anchor-drift
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
related_code:
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/core_runtime_state.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
tests:
  - cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked
resolved_at: 2026-07-14
---


# Runtime15：core runtime state 守卫要求已迁出的 plugin bridge lifecycle 字段

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：M1 测试阶段 / exact core-min scene gate
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：失败测试与被校验的 core runtime state owner 均属于 Runtime15 命名/结构守卫；Editor02 的 world generation 与 inspection 不拥有该字段迁移。

## 失败现象与复现证据

协调器管理的 Windows job `c9e5489e32cd4bf48dd5f69dacda5261` 执行：

```text
cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked
```

测试目标成功编译并执行 `594` 个匹配项，其中 `590 passed / 4 failed`。外部失败
`runtime_15_core_runtime_state_module_uses_owner_name` 要求
`core/runtime/state/core_runtime_state.rs` 含 `plugin_bridge_lifecycle`，但当前 owner 不再包含该字段；
`core_runtime_state.rs` 的新鲜源码扫描同样找不到该锚点。其余三项失败属于 Editor02
generation 原子计数，已在来源计划内单独修复。

## 最低共享层根因

最低已证明边界是 Runtime15 core runtime state 命名守卫与当前 lifecycle owner 形状脱节。
守卫仍把 `plugin_bridge_lifecycle` 当作 `CoreRuntimeInner` 的必备字段，而当前实现已经把该责任
迁往 core handle/plugin bridge lifecycle 边界；测试没有同步新的唯一 owner，也没有明确验证旧字段已删除。

## 架构修复验收

- Runtime15 确认 plugin bridge lifecycle 当前唯一 owner，并让守卫读取该 owner 的真实字段/入口。
- `core_runtime_state.rs` 只保留当前职责；不得为满足字符串守卫恢复重复字段或兼容 façade。
- 守卫明确断言退役 owner 形状不会回流，同时保留 `core_runtime_state.rs` 的当前命名/模块边界检查。
- 原 exact core-min 命令不再失败于 `runtime_15_core_runtime_state_module_uses_owner_name`。

## 禁止临时方案

- 不在 `core_runtime_state.rs` 添加无行为字段、注释字符串、alias 或测试专用锚点。
- 不放宽或删除整个 Runtime15 命名守卫来隐藏 owner 漂移。
- 不把该结构失败归为 Editor02 generation 功能失败。

## 修复结果与回传

- 根因：Runtime15 naming guard still required the retired plugin_bridge_lifecycle field after lifecycle ownership moved to the neutral runtime module observer boundary.
- 架构修复：The guard now verifies CoreRuntimeInner.runtime_module_lifecycle_observer, CoreHandle lock ownership, and the CoreRuntime install facade while rejecting plugin-specific lifecycle state names from all three core owners.
- 验证：Fresh standalone owner guard 1/1 and child-budget guard 1/1 passed; full structure convention passed 1304/1304; managed Windows job 1d651b687cf647fe8498321d7095c731 ran cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked with 596 passed and 0 failed.
- 回传：The core runtime state guard now follows the neutral observer hard cut and the original Editor02 upward gate is green.
