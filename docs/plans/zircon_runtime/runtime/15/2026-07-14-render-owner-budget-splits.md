---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_command_lists.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_recording.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests/cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests/advanced_materials.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/sprite_stage_selection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/pipeline_resource_usage.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_recording.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests/cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests/advanced_materials.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/sprite_stage_selection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/pipeline_resource_usage.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_pass_gpu_context_mesh_command_lists.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/render_mesh_draw_command_list.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/render_structure.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_pass_gpu_context_mesh_command_lists.rs::runtime_15_render_pass_gpu_context_mesh_command_lists_are_child_owner
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/render_mesh_draw_command_list.rs::runtime_15_mesh_draw_command_list_is_folder_backed
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/render_structure.rs::review_f16_compiled_scene_render_path_uses_split_owners
  - rustfmt --edition 2021 --check
  - git diff --check
doc_type: milestone-detail
---

# Runtime 15 Render Owner 预算拆分

## 状态与完成项目

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M3 | Render18 增量后的三个最低 owner 预算恢复 | `runtime_15_render_owner_budget_split_current_source_managed_build_passed` | 2026-07-14 | GPU context 387/146/516 行；mesh test owners 385/81/54 行；compiled render owners 390/110/18 行。三项结构守卫 3/3 通过；Frameworks05 已 fixed 回传范围外 UI text manager-access 漂移，fresh managed default-feature Runtime build 最终通过。 |

## Owner 边界

- `gpu.rs` 只保留 `RenderPassGpuExecutionContext` 数据、构造、context access 和非 mesh recording 路由。
- `gpu/mesh_command_lists.rs` 继续拥有 command-list DTO、phase streams 和 HZB candidate counters。
- `gpu/mesh_recording.rs` 完整拥有 depth prepass、shadow atlas、standard/advanced/transmission mesh stage 与 TAA reactive-mask 录制。
- `mesh_draw_command_list/tests.rs` 保留共享夹具与基础排序、统计、phase 构建和 variant 覆盖；`tests/cache.rs` 与 `tests/advanced_materials.rs` 分别拥有 cache lifecycle 和 advanced PBR/transmission ordering 覆盖。
- `render/render.rs` 保留 compiled-scene orchestration；`render/sprite_stage_selection.rs` 拥有 sprite stage selection 与局部测试夹具；`render/pipeline_resource_usage.rs` 拥有所有 graph feature 共用的 resource-write predicate。
- 所有迁移均使用当前路径硬切；没有兼容模块、re-export shim、重复实现或预算放宽。

## 验证

- 红态证据：旧 current-source guard 对 `gpu.rs` 报 897 行，超过 800 行预算；同步 inventory 记录 mesh tests 515 行、F16 render 510 行。
- scoped `rustfmt --edition 2021` 已执行并通过复查。
- current-source 行数：GPU root/list/recording = 387/146/516；mesh test root/cache/advanced = 385/81/54；compiled render root/sprite/resource-usage = 390/110/18。
- 三项守卫要求新 child mount、迁移锚不回流、模块文档登记和 focused budgets。
- current-source standalone structure harness 已 3/3 通过；收口复验由协调器临时 test lane `6ee2d5f2a9c34e7e80c1338b5887dd3a` 执行，三项 exact test 均为 1/1、exit 0。
- 首次共享托管 Cargo 与第二次显式目标请求均在 Rust 编译前被同一外部兼容池占用拒绝。随后 managed ephemeral lane `ff2fa0c62ede4e858ef24e01382c4263` 完整编译依赖并进入 default-feature `zircon_runtime`；本切片 owner 没有产生诊断，build 在 `graphics/scene/scene_renderer/ui/construct.rs:81` 暴露范围外 ProjectAssetManager hard-cut 类型漂移（E0308/E0277）。该问题按 Failure 流程交由 Frameworks05 在最低真实 use point 修复，并已回传 [`fixed-2026-07-14-ui-text-project-asset-manager-access-consumer-drift.md`](fixed-2026-07-14-ui-text-project-asset-manager-access-consumer-drift.md)。最终 managed Windows job `9dac70c034fb4aa18155d370f77073e1` 执行 `cargo build -p zircon_runtime --locked` 通过，耗时 7m51s。

## 未关闭范围

Runtime15 总计划仍为 `in_progress`。本记录仅关闭 Render18 增量触发的三个 owner 预算问题，不代表完整 runtime architecture、全 workspace 测试或全部 Runtime05-15 已完成。
