---
handoff_kind: fixed
status: fixed
created_at: 2026-07-17
resolved_at: 2026-07-17
summary_slug: compiled-pipeline-frame-derived-recomputation
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/render/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_enabled_features.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline/runtime_feature_flags.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline/resource_write_index.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline/runtime_metadata.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/plugin_features.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/render_structure.rs
tests:
  - render01_compiled_pipeline_runtime_metadata_builds_resource_write_index_once_for_scaled_graphs
  - render01_compiled_pipeline_runtime_metadata_freezes_descriptor_capability_flags
  - render01_compiled_pipeline_executor_validation_cache_skips_stable_10_100_500_pass_rescans
  - render01_compiled_pipeline_executor_revoke_invalidates_cache_before_submission
  - render01_compiled_pipeline_cached_sources_are_immutable_and_frame_flags_are_precomputed
  - plugin_feature_buffer_minimum_size_survives_graph_resource_planning
---


# Render01：compiled pipeline 派生状态在稳态帧重复计算

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F2 render-frame/compiled-scene/registry validation 静态审查
- 修复责任计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 共同责任：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 交接原因：派生状态应成为 compiled pipeline/RDG artifact；帧执行器本地加 memoization 会绕过插件 hot-reload generation。

## 失败现象与复现证据

每个 `render_frame_with_pipeline_to_target` 都调用 `runtime_features_from_pipeline`；该函数为多个 builtin/plugin/capability flag 分别线性扫描 `enabled_features`。同一入口又为 SSR/HZB/exposure/volumetric history 多次调用 `pipeline_writes_resource`，每次遍历全部 live pass 与 resource access。

随后 `render_compiled_scene` 每帧调用 `RenderPassExecutorRegistry::validate_compiled_pipeline`：遍历全部 live pass，为每个 executor id clone `String`、构造 `RenderPassExecutorId` 并查 registry。实际 pass execute 时还会再次查 executor。compiled pipeline 在稳态不变，这些都是重复派生工作。

## 最低共享层根因

`CompiledRenderPipeline` 只保存原始 Vec/graph，没有冻结 frame runtime flags、resource-usage bitset 或 validation generation；`RenderPassExecutorRegistry` 也没有可用于 pipeline validation cache 失效的 generation。

## 架构修复验收

- compile/install 时生成紧凑 runtime feature flags 与 named resource usage bitset，frame path O(1) 读取。
- executor registry 每次 register/revoke/reload 增 generation；pipeline validation 缓存绑定 `{pipeline generation, registry generation}`，稳态不遍历/不 clone。
- plugin hot reload/revoke 后下一次 submit 在编码 GPU 命令前重新验证，missing executor 保持 typed failure；不能因缓存接受 stale executor。
- 加 compute-count/alloc benchmark，覆盖 10/100/500 pass 与 stable/hot-reload 两种路径。

## 禁止临时方案

- 不得直接删除 executor validation 或把错误延迟到半帧 execute 后。
- 不得只按 pipeline handle 缓存而忽略 executor registry generation。

## 修复结果与回传

- 根因：CompiledRenderPipeline lacked frozen frame-derived metadata and RenderPassExecutorRegistry lacked generation-bound validation caching.
- 架构修复：Freeze runtime feature flags and resource-write indices at from_parts construction; bind successful executor validation to pipeline and registry generations with mutation invalidation.
- 验证：Managed source-bound jobs 9cfa78c7/3a001e66 (5 passed, 0 failed) and 80390edb/c08a17d5 (exact 1 passed, 0 failed) on fingerprint 786c6aed; both exit 0 and released.
- 回传：Performance compiled-scene frame path may resume; Plan01 hard cut proceeds to independent review and managed milestone closeout.
