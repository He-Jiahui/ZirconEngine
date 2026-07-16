---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: ecs-resource-marker-owner-missing
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
related_code:
  - zircon_runtime/src/scene/ecs/resource/mod.rs
  - zircon_runtime/src/scene/tests/component_structure/runtime_08_owner_tree.rs
  - zircon_runtime/src/scene/tests/component_structure/runtime_world_domains.rs
tests:
  - cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked
  - zircon_runtime-85ffd78eca5b57a4.exe scene::tests::component_structure::runtime_08_owner_tree::runtime_08_ecs_data_owner_trees_stay_folder_backed_after_cutover --exact --nocapture
  - zircon_runtime-85ffd78eca5b57a4.exe scene::tests::component_structure::runtime_world_domains::scene_components_keep_only_runtime_world_domains_after_editor_boundary_cutover --exact --nocapture
resolved_at: 2026-07-14
---


# Runtime08：ECS resource marker owner 缺失

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：M1 测试阶段 / Shader04 broad-gate 阻断后的 supplemental minimal scene self-check
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 交接原因：两个 scene 结构守卫收敛到同一 Runtime08 ECS resource owner 文件缺失；该边界低于且独立于 Editor02 world-sync/inspection 变更。

## 失败现象与复现证据

在协调器 job `4538bbef2c6c4f24a949fb138b21b00a`、Windows target `E:\cargo-targets\zircon-engine\pool\569e0d4b772933e1ab3d593a42ad81224230969e2a1138986c1f66f825584999` 执行：

```text
cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked
```

该 supplemental gate 编译成功并实际运行 586 个匹配测试；Editor02 新增的 world generation、split inspection、subtree hash 与 authoring-boundary 测试均通过，但以下两个既有结构守卫失败：

1. `runtime_08_ecs_data_owner_trees_stay_folder_backed_after_cutover`：`Runtime 08 ECS data owner resource should keep child marker.rs`。
2. `scene_components_keep_only_runtime_world_domains_after_editor_boundary_cutover`：`expected scene ECS module ecs/resource/marker.rs`。

使用生成的 lib-test binary 对两项分别 `--exact --nocapture` 复现，均为 0 passed / 1 failed；工作区当前 `zircon_runtime/src/scene/ecs/resource/` 仅有 `id.rs`、`registry.rs`、`mod.rs`，`marker.rs` 不存在。

## 最低共享层根因

Runtime08 的 ECS resource owner tree 与其两道结构守卫未收敛：`resource/mod.rs` 目前直接把 `crate::core::framework::scene::SceneResource` 重导出为 `Resource`，但既有硬切守卫要求 resource marker 使用明确的 `ecs/resource/marker.rs` child owner。当前证据只证明 owner-tree 缺口，不替 Runtime08 决定 marker 应做类型别名、trait owner 还是受控 re-export；该架构选择必须由 Runtime08 按无兼容层规则定稿。

## 架构修复验收

- `zircon_runtime/src/scene/ecs/resource/marker.rs` 成为唯一、明确的 ECS resource marker owner，`resource/mod.rs` 只保留结构声明与导出。
- 不在 `mod.rs`、调用点或测试中复制 `Resource` 契约；core scene resource 与 ECS marker 的所有权方向由 Runtime08 文档说明。
- 两个精确结构守卫分别通过 1/1。
- `cargo test -p zircon_runtime --lib scene:: --no-default-features --features core-min --locked` 通过后回传 Editor02；默认特性正式门禁仍需同时等待 Shader04 编译交接返回。

## 禁止临时方案

- 不删除或放宽结构守卫，不创建空占位文件，不增加兼容别名/shim/重复 trait 真相。
- 不把 `marker.rs` 仅作为测试路径满足器；它必须承载经 Runtime08 定稿的真实 owner 边界。
- 不由 Editor02 会话越权修改 Runtime08 ECS resource 核心契约。

## 修复结果与回传

- 根因：The neutral SceneResource trait moved to core/framework, but scene/ecs/resource/mod.rs retained the scene vocabulary export while Runtime08 folder-backed ownership guards required a real marker.rs leaf owner.
- 架构修复：Added scene/ecs/resource/marker.rs as the controlled Resource vocabulary export of the single neutral SceneResource trait; kept resource/mod.rs structural and updated ECS owner documentation without aliases or duplicate traits.
- 验证：Scoped rustfmt and diff checks passed; managed core-min scene gate compiled and ran 586 matches; Runtime08 resource owner-tree guard passed, and the remaining SystemStage guard drift was handed to Runtime02.
- 回传：Runtime08 resource marker ownership is repaired and returned to Editor02; Editor02 can drop this blocker while still waiting on Shader04 and Runtime02 upward gates.
