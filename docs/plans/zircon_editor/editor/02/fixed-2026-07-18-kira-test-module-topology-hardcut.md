---
handoff_kind: fixed
status: fixed
created_at: 2026-07-17
summary_slug: kira-test-module-topology-hardcut
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
origin_workflow_node: M1
fixing_plan: docs/plans/zircon_plugins/02-sound.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_plugins/02
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/sound/runtime/src/tests/kira_bridge/graph/routing.rs
  - zircon_plugins/sound/runtime/src/tests/kira_graph_sync.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/validation.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/validation/support.rs
tests:
  - cargo +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sound_runtime --locked
resolved_at: 2026-07-18
---


# Sound02：Kira hard-cutover 测试模块拓扑未闭合

## 来源执行者

- 来源计划：docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
- 来源执行者：plugins02-sound-m1-kira-core-20260717
- 来源执行切片：M1 上游恢复门禁中的 Sound plugin gate
- 修复责任计划：docs/plans/zircon_plugins/02-sound.md
- 交接原因：Kira hard-cutover 后的测试模块可达性属于 Sound M1 的最低测试边界，Editor02 不并行修改 Sound 测试树。
- 受管证据：job `65ab95c6b8524567b7056bfd9fab5a8b` / run `d6ef23e43d8d47dbbd2453520b23bbd3`。

## 失败现象与复现证据

当前源码已越过先前 18 个 Kira 0.12 生产 API 编译错误，但完整 plugin lib-test 在测试模块加载阶段以 7 个 `E0432` 终止，尚未执行测试。

- `mixer_graph/sends/crud/routing.rs` 只声明了 pre/post-effect tap 测试模块，缺失它们共享的 `tap_mix` 模块边界；
- `mixer_graph/support.rs` 将同级 `sidechain` 解析为自身子模块，实际 canonical owner 是父级 mixer-graph module；
- output-device catalog 的 backends/devices 子模块引用 `super::support`，但共享 software-null fixture 不在该可见性边界内。

这是一处 hard-cutover 后的测试模块图收束问题。不得通过删除测试、放宽 `--locked`、改为只跑生产 lib，或将 helper 复制到各测试文件来掩盖。

## 最低共享层根因

测试模块的 facade、shared helper 与目录实际所有权在 hard-cutover 时不同步：routing 的 shared tap helper、
sidechain helper 和 output-device catalog fixture 都失去唯一可达入口。它是模块图问题，而非生产 lib 或
Cargo lock 问题。

## 架构修复验收

- 为 track-send tap、sidechain 和 software-null fixture 建立唯一、目录匹配的模块声明与可见性边界；
- 保持现有测试名称和 test helper 的单一实现，不复制 fixture；
- 先增加或更新针对模块入口/fixture 可达性的 focused regression，再重跑原始完整受管 plugin gate，必须实际进入测试阶段并报告 test count；
- 与 Kira API 迁移、graph-sync 失败一起由同一 broad Sound owner 审查、验证并写入完整 immutable manifest。

## 禁止临时方案

- 不得移除失败的 test modules、在调用点内联 software-null helper，或以 `cfg` 跳过它们；
- 不得由 F2、Shader06、Editor01 或 Performance owner 修补 Sound 测试树。

## 修复结果与回传

- 根因：The hard cut deleted tap_mix, sidechain and output catalog support modules while lib-test declarations still imported those retired module paths.
- 架构修复：Rewired the tests to the current named routing, mixer-graph and output-device support owners and removed the stale module declarations instead of restoring compatibility shims.
- 验证：The current-source Sound library now compiles; route focused passed 8/8, final focused passed 1/1, plugin broad passed 344/344, and package check exited 0.
- 回传：The stale Sound test module topology is fixed without restoring deleted hard-cut paths; immutable M1 milestone SHA remains the downstream acceptance boundary.
