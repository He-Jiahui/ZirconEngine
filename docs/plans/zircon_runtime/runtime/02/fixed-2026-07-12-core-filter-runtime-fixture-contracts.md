---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
summary_slug: core-filter-runtime-fixture-contracts
origin_plan: docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
fixing_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
origin_child_dir: docs/plans/zircon_runtime/runtime/02
fixing_child_dir: docs/plans/zircon_runtime/shader/06
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_environment_ibl_graph_resources.rs
tests:
  - cargo test -p zircon_runtime --lib --locked --offline --jobs 1 core:: -- --test-threads=1
  - cargo test -p zircon_runtime --lib --locked --offline --jobs 1 graphics::scene::scene_renderer::core::runtime_features::runtime_features_from_pipeline::tests::pluginized_rendering_feature_names_drive_runtime_post_process_flags -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked --offline --jobs 1 graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene::render::bind_environment_ibl_graph_resources::tests::environment_ibl_source_binder_preserves_missing_required_source_when_frame_has_no_view -- --exact --test-threads=1
resolved_at: 2026-07-12
---


# Shader 06：Runtime `core::` 门中的渲染测试夹具契约滞后

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md`
- 来源执行切片：Runtime 02 当前源码综合 `core::` 回归门
- 修复责任计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 交接原因：两项失败均位于 Shader 06 活跃拥有的 `scene_renderer/core` 测试夹具；Runtime 02 只负责暴露综合回归，不越界修改正在收尾的 IBL/PBR owner。

## 失败现象与复现证据

2026-07-12 在 Windows retained target 的当前 Runtime lib test binary 上执行 `core::` 过滤，共运行 675 项，结果为 `669 passed / 6 failed`。其中四项已归入 Runtime 09 与渲染测试夹具本地修复；本交接保留以下两项 Shader 06 owner 失败：

1. `graphics::scene::scene_renderer::core::runtime_features::runtime_features_from_pipeline::tests::pluginized_rendering_feature_names_drive_runtime_post_process_flags` 在 `compile(...).unwrap()` 处失败。测试为 `plugin-ssao-runtime-flag` 声明 `QueueLane::AsyncCompute`，却没有声明 `RenderGraphComputeWorkload`；当前编译器正确拒绝无 compute workload 的异步计算 pass。
2. `graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene::render::bind_environment_ibl_graph_resources::tests::environment_ibl_source_binder_preserves_missing_required_source_when_frame_has_no_view` 在 transient materialization 阶段提前返回错误：`render graph materialization missing 1 required external resource bindings: external texture environment.ibl.source_cubemap`。测试仍假设可以先 materialize，再由后续 validation report 表达缺失 required external，已落后于当前严格 materialization 契约。

对两项测试分别按完整测试名精确重跑，失败可稳定复现；没有生产 Runtime 02 core/root/generated 测试失败。

## 最低共享层根因

Shader 06 的两个测试夹具没有随当前渲染图契约同步：异步计算 fixture 缺失唯一 compute workload；IBL required-external fixture 仍使用旧的“materialize 成功、稍后报告缺失”顺序。最低修复层是各自的 owner-local fixture/断言，不应放宽渲染图编译与 required-external 校验。

## 架构修复验收

- SSAO plugin fixture 为 `AsyncCompute` pass 声明真实、唯一且与当前 SSAO owner 一致的 compute workload；不得改成 Graphics lane 规避编译契约。
- IBL required-external 测试按当前 materialization/validation 责任边界断言缺失 source cubemap；不得把 required external 降级为 optional 或静默注入占位资源。
- 两项完整测试名精确执行均通过。
- Runtime 02 重新执行 `core::` 综合过滤门，并以测试框架汇总更新编号产出归档。

## 禁止临时方案

- 禁止添加别名、兼容 shim、静默 fallback、重复真相、test-only 绕过或单调用点特例。
- 禁止放宽 compute workload、required external 或测试验收条件来隐藏失败。

## 修复结果与回传

- 根因：Shader 06 test fixtures lagged strict async-compute workload and required-external validation contracts.
- 架构修复：SSAO fixture now declares the production viewport compute workload; IBL fixture asserts the strict missing required external texture error without fallback.
- 验证：Fresh default-feature Runtime lib no-run exited 0; both full-name exact tests passed 1/1.
- 回传：Returned verified Shader 06 fixture contract repair to Runtime 02 for core-filter rerun.
