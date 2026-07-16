---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
resolved_at: 2026-07-15
summary_slug: depth-prepass-source-guard-owner-drift
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_recording.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/tests.rs
tests:
  - cargo test -p zircon_runtime --lib graphics::scene::scene_renderer::graph_execution::render_pass_execution_context::gpu::tests::depth_prepass_binds_forward_shadow_receiver_layout_slot --locked -- --exact --nocapture
  - cargo test -p zircon_runtime --lib scene:: --locked
---


# Runtime15：depth prepass source guard 未随 owner 拆分迁移

## 产出记录与时间

| 状态 | 日期 | 完成项目与验收证据 |
|---|---|---|
| `FIXED / RETURNED TO EDITOR02` | 2026-07-15 | Runtime15 将 source guard 硬切到 canonical `gpu/mesh_recording.rs` owner，未恢复父 `gpu.rs` wrapper 或兼容重导出；exact job `aa6beee8c42c455e94547d1dc41c1cd2` 为 `1 passed / 0 failed`，依赖 Render18 收敛后的上行 scene job `2d3753edcc2149799ffb0e88eb31b6d3` 为 `1667 passed / 0 failed / 6 ignored`，Failure 已按 lifecycle 回传 Editor02 M1.3。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：Editor02 M1 fresh 默认特性 runtime scene 验收门禁
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：失败来自 Runtime15 完成 GPU recording owner 拆分后遗漏的 source guard 迁移；Editor02 不拥有 Render graph 模块布局或结构测试。
- 受管证据：Windows job `acfc6c19219441e498a6af33ce4b5e7a`，日志 `E:\ZirconBuilds\editor02-m1-runtime-scene-final-20260714.log`。

## 失败现象与复现证据

默认 scene 门禁运行 1709 项，结果 `1700 passed / 3 failed / 6 ignored`。其中 `depth_prepass_binds_forward_shadow_receiver_layout_slot` 在 `gpu/tests.rs:5` 失败，因为测试仍以 `include_str!("../gpu.rs")` 搜索 `record_depth_prepass_to_resources`。Runtime15 已把 GPU mesh recording 硬切到 `gpu/mesh_recording.rs`；三个受测符号当前都存在于新 owner，父 `gpu.rs` 不再拥有实现。

## 最低共享层根因

生产 owner 拆分已经完成，但同一 Runtime15 结构切片的 source guard 没有迁到新 child owner。失败是测试 owner 与 folder-backed 模块布局不一致，不是 Editor02 world-sync 或 depth prepass 行为缺失。

## 架构修复验收

- 守卫从 canonical `gpu/mesh_recording.rs` owner 验证三个符号，或迁入该 child 的 folder-backed tests；不得继续要求父 facade 拥有实现。
- 保持 `gpu.rs` 为窄 facade，不把 recording 函数重新内联或兼容重导出回父文件。
- exact source guard 通过，并 fresh 重跑 Editor02 的默认 scene 门禁确认无新的 Render/Runtime15 失败。

## 禁止临时方案

- 禁止复制符号文本、注释锚或空 wrapper 到 `gpu.rs` 取悦字符串测试。
- 禁止恢复超预算父 owner、兼容模块或双路径实现。
- 禁止在 Editor02 过滤该测试或把 source guard 改为无条件通过。

## 修复结果与回传

- 根因：Runtime15 moved depth-prepass recording into gpu/mesh_recording.rs but the source guard continued reading the parent gpu.rs facade.
- 架构修复：Moved the guard source owner to canonical gpu/mesh_recording.rs while keeping the parent facade narrow and without wrappers or compatibility re-exports.
- 验证：Exact job aa6beee8c42c455e94547d1dc41c1cd2 passed 1/0; after dependent Render18 layout convergence, managed upward scene job 2d3753edcc2149799ffb0e88eb31b6d3 passed 1667/0 with 6 ignored.
- 回传：The Runtime15 source-owner guard and originating Editor02 scene gate are green on current source; return this failure to Editor02.
