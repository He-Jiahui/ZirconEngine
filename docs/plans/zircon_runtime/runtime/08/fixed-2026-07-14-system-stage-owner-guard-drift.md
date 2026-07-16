---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: system-stage-owner-guard-drift
origin_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
fixing_plan: docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
origin_child_dir: docs/plans/zircon_runtime/runtime/08
fixing_child_dir: docs/plans/zircon_runtime/runtime/02
related_code:
  - zircon_runtime/src/core/framework/scene/system_stage.rs
  - zircon_runtime/src/scene/ecs/mod.rs
  - zircon_runtime/src/scene/tests/component_structure/runtime_world_domains.rs
tests:
  - cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked
  - zircon_runtime-85ffd78eca5b57a4.exe scene::tests::component_structure::runtime_world_domains::scene_components_keep_only_runtime_world_domains_after_editor_boundary_cutover --exact --nocapture
resolved_at: 2026-07-14
---


# Runtime02：SystemStage owner 硬切后结构守卫漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 来源执行切片：Editor02 `ecs-resource-marker-owner-missing` Failure 上行验证
- 修复责任计划：`docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md`
- 交接原因：Runtime08 resource marker 的两道原始结构失败已经收敛；上行 scene gate 暴露的剩余失败属于 Runtime02 已提交的 core/framework 吸收切换，而不是 ECS resource owner。

## 失败现象与复现证据

在协调器管理的 Windows test lane `E:\cargo-targets\zircon-engine\pool\569e0d4b772933e1ab3d593a42ad81224230969e2a1138986c1f66f825584999` 执行：

```text
cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked
```

编译成功并运行 586 个匹配测试。Runtime08 的 `runtime_08_ecs_data_owner_trees_stay_folder_backed_after_cutover` 已通过；`scene_components_keep_only_runtime_world_domains_after_editor_boundary_cutover` 仍失败，精确诊断为缺少 `scene/ecs/system_stage.rs`。

当前 `HEAD=facb719f4da98953ec83f682175389916da51b6b` 的提交 `refactor: converge runtime architecture and engine tooling` 明确删除了 `zircon_runtime/src/scene/ecs/system_stage.rs`，并将 `scene/ecs/mod.rs` 改为从 `core::framework::scene::SystemStage` 导出。结构守卫仍要求旧文件存在，未随该硬切更新。

## 最低共享层根因

最低已证明边界是 Runtime02 的 core/framework scene owner 吸收与 scene 结构守卫之间的同提交漂移：生产所有权已迁走，测试仍把已删除的旧 ECS owner 当作必需文件。这里不应恢复旧文件，也不应在 ECS 下建立兼容 re-export owner。

## 架构修复验收

- `SystemStage` 的唯一声明 owner 保持在 `core/framework/scene/system_stage.rs`，`scene/ecs/mod.rs` 仅保留受控导出。
- `runtime_world_domains.rs` 断言新 owner 存在并断言旧 `scene/ecs/system_stage.rs` 不复活。
- 精确结构守卫通过 1/1。
- 原 Runtime08 上行命令 `cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked` 通过，随后回传 Runtime08/Editor02。

## 禁止临时方案

- 不恢复 `scene/ecs/system_stage.rs`，不添加 alias、compatibility shim、重复类型或空占位文件。
- 不删除或放宽整个结构守卫；只把它迁到当前真实 owner 与退役路径断言。
- 不把测试漂移误归为 Runtime08 resource marker 实现失败。

## 修复结果与回传

- 根因：Runtime02 已将 SystemStage 唯一声明 owner 硬切到 core/framework/scene，但 runtime_world_domains 结构守卫仍要求已删除的 scene/ecs/system_stage.rs。
- 架构修复：守卫改为要求 core/framework/scene/system_stage.rs 存在、旧 scene/ecs/system_stage.rs 不存在，并验证 scene/ecs/mod.rs 仅从新 owner 受控导出；未恢复兼容文件。
- 验证：Windows 受管 job 1d651b687cf647fe8498321d7095c731：cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked，exit 0，596 passed / 0 failed；当前文件审计确认新 owner 存在且旧 owner 不存在。
- 回传：Runtime02 SystemStage owner 守卫漂移已按硬切架构修复并通过 Editor02 上行 core-min 场景门禁，回传 Runtime08/Editor02。
