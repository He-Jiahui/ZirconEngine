Plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
Milestone: M1
Status: pending
Files: ["docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md", "docs/plans/zircon_editor/editor/02/2026-07-14-world-sync-m1-output-records.md", "docs/plans/zircon_editor/editor/02/fixed-2026-07-14-advanced-pbr-transparent-selection-uninitialized.md", "docs/plans/zircon_editor/editor/02/fixed-2026-07-14-cargo-release-retains-live-child-process-lock.md", "docs/plans/zircon_editor/editor/02/fixed-2026-07-14-compute-fullscreen-descriptor-compile-regression.md", "docs/plans/zircon_editor/editor/02/fixed-2026-07-14-core-runtime-state-plugin-bridge-lifecycle-anchor-drift.md", "docs/plans/zircon_editor/editor/02/fixed-2026-07-14-ecs-resource-marker-owner-missing.md", "docs/plans/zircon_editor/editor/02/fixed-2026-07-14-editor-retained-host-manager-resolver-consumer-drift.md", "docs/plans/zircon_editor/editor/02/fixed-2026-07-14-f18-asset-manager-review-guard-owner-drift.md", "docs/plans/zircon_editor/editor/02/fixed-2026-07-14-level-manager-name-core-error-import-drift.md", "docs/plans/zircon_editor/editor/02/fixed-2026-07-14-mutation-queue-finish-lease-stall.md", "docs/plans/zircon_editor/editor/02/fixed-2026-07-14-mutation-queue-offline-recurrence.md", "docs/plans/zircon_editor/editor/02/fixed-2026-07-14-project-asset-manager-access-test-consumer-drift.md", "docs/plans/zircon_editor/editor/02/fixed-2026-07-14-runtime-diagnostics-pane-payload-visibility-drift.md", "docs/plans/zircon_editor/editor/02/fixed-2026-07-14-standard-pbr-transmission-render-queue-root-export-drift.md", "docs/plans/zircon_editor/editor/02/fixed-2026-07-14-ui-text-module-split-import-drift.md", "docs/plans/zircon_editor/editor/02/fixed-2026-07-14-vm-reflection-catalog-test-support-import-drift.md", "docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md", "docs/plans/zircon_runtime/runtime/08/fixed-2026-07-14-system-stage-owner-guard-drift.md", "docs/zircon_runtime/scene/ecs.md", "docs/zircon_runtime/scene/inspection.md", "docs/zircon_runtime_interface/world_sync.md", "zircon_runtime/src/scene/ecs/resource/marker.rs", "zircon_runtime/src/scene/ecs/resource/mod.rs", "zircon_runtime/src/scene/inspection/snapshot.rs", "zircon_runtime/src/scene/inspection/tests.rs", "zircon_runtime/src/scene/tests/component_structure/runtime_world_domains.rs", "zircon_runtime/src/scene/world/error.rs", "zircon_runtime/src/scene/world/generation.rs", "zircon_runtime/src/scene/world/generation/tests.rs", "zircon_runtime/src/scene/world/records.rs", "zircon_runtime/src/scene/world/typed_api.rs", "zircon_runtime/src/scene/world/typed_api/fixed_components.rs", "zircon_runtime_interface/src/tests/world_sync_contracts.rs", "zircon_runtime_interface/src/world_sync/query.rs"]

# Editor02 WorldSyncProtocol M1 里程碑清单

## Scope Delivered

M1 已完成传输中立的 world-sync DTO、查询世代提示、稳定排序、runtime-only 世界世代号、结构变更与固定组件变更打点、拆分 inspection 入口、迭代式 5k 深层级与环边指纹，以及对应 ECS owner 硬切依赖。旧 owner、兼容 DTO、别名入口与递归层级实现均未保留。

当前清单仍为 `pending`：Text01、Shader04 与 Plugins08 原 Failure 均已 fixed 回传；fresh 默认 scene 门禁确认全部 Editor02 M1 合同通过，但暴露 Runtime15 depth-prepass source guard、Text05 CJK SDF loaded-font 统计与 Plugins08 VM 动态属性写入结构复发三项外部 Failure。三项未回传前，此文件不得用于绑定不可变 milestone manifest。

## Fresh Testing Evidence

- `cargo test -p zircon_runtime_interface --locked`：协调器 job `97771db615b34895b9b3e1956b1aa4c5`，252 passed / 0 failed，doc-tests 0/0。
- `cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked`：协调器 job `1d651b687cf647fe8498321d7095c731`，596 passed / 0 failed。
- `cargo test -p zircon_runtime --lib scene:: --locked`：prior Failure 回传后的 managed job `acfc6c19219441e498a6af33ce4b5e7a` 以单 worker 完成 19m03s 冷编译并自然运行 1709 项，结果 1700 passed / 3 failed / 6 ignored / 6354 filtered out；全部 Editor02 M1 测试通过。三条外部失败已导入 Runtime15、Text05、Plugins08 的新 lifecycle；完整日志 `E:\ZirconBuilds\editor02-m1-runtime-scene-final-20260714.log`。
- scoped `rustfmt --check`、`git diff --check` 已通过；本轮 Failure 与 plan-output 审计在三个 artifact 写入后重跑，外部既有计划问题不计为 Editor02 通过。

## Review

Noether 的两轮独立代码复核已关闭 lifecycle dirty 时序与 cycle-edge hash 两项 Important，第二轮结论为 0 Critical / 0 Important / 0 Minor。协调器要求的独立 Session 审查必须在最终默认场景门禁和清单指纹固定后重新提交，当前证据不提前绑定。

## 产出记录与时间

| 时间 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- |
| 2026-07-14 | fresh 默认 scene 已验证 Editor02 自有合同；三个外部 Failure 待返回；清单未绑定 | prior fixed 均已返回；job `acfc6c19219441e498a6af33ce4b5e7a` 自然汇总 1700/3/6，三条失败已分别路由 Runtime15、Text05、Plugins08，未混入 M1 提交清单。 | 三项 Failure 回传后 fresh 跑默认场景门禁；通过后改 `Status: completed`，再 claim/attribute、prepare、validate、独立 Session review、milestone commit。 |
