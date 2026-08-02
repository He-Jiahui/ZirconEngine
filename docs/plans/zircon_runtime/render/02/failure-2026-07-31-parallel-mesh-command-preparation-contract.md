---
handoff_kind: failure
status: open
created_at: 2026-07-31
summary_slug: parallel-mesh-command-preparation-contract
origin_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
fixing_plan: docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
origin_child_dir: docs/plans/zircon_runtime/render/17
fixing_child_dir: docs/plans/zircon_runtime/render/02
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
tests:
  - managed current-source zircon_runtime mesh command tests
  - render_perf_parallel_prepare_deterministic_sort
  - Render17 PF-M2 current-source runtime validation
---

# Render02: parallel mesh command preparation contract

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 来源执行切片：PF-M2 prepare/queue rayon 并行
- 修复责任计划：`docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md`
- 交接原因：Mesh pass command artifact、variant id 分配和 cached command 生命周期由 Render02 拥有；Render17 只能消费其确定性、可并行的准备边界。

## 失败现象与复现证据

Render17 对当前源码的静态审查确认：`build_mesh_pass_command_buffers_cached` 对每个 batch 同时持有可变 `MeshPipelineVariantResolver` 与 `CachedMeshDrawCommands`。variant registry 在 miss 时分配递增 id 并写入 miss report，cache lookup/miss/store 也在同一转换循环中修改唯一 owner。直接将 processor 或 batch 循环放入 rayon 会导致非确定性 id/merge 顺序；在 worker 中包 mutex 会把高频路径重新串行化，违反 PF-M2 的并行 prepare 目标。

## 最低共享层根因

Render02 尚未提供“owner-thread variant/cache transaction + immutable prepared batch input + ordered command chunk merge”的命令构建契约。现有 `MeshPassBuildContext` 直接借用 `&mut MeshPipelineVariantResolver`，使 processor 输出同时承担纯 command 生成、variant 注册和跨帧 cache 写入三种职责，不能安全交给共享 `TaskPool`。

## 架构修复验收

- Render02 定义由 owner 预解析的 variant/cache transaction 或等价快照，使 worker 只消费不可变 batch 数据和已稳定的 variant id。
- worker 通过调用方提供的 `TaskPool` 并行准备独立 command chunk；不得在渲染模块新建 `ThreadPoolBuilder`。
- command 与 cache mutation 按 source draw index、phase 和既有 sort key 的规范顺序单点合并，serial 与 parallel 的 command 序列、cache hit/miss/rebuild 统计完全一致。
- 加入命令序列和 cache-stat parity 测试，并通过 Render02 managed current-source mesh gate；随后由 Render17 重跑 PF-M2 runtime gate。

## 禁止临时方案

- 不得将 `MeshPipelineVariantRegistry` 或 `CachedMeshDrawCommands` 包入每 batch/processor 的 mutex 后宣称并行。
- 不得在 Render17 创建第二套 variant registry、cache 或 command artifact，也不得改变 graph/executor 顺序来规避合并。
- 不得把不确定的 variant id、cache 统计或 command 顺序放宽为测试容忍项。

## 修复结果与回传

Open state: `实现已落地，待 current-source managed validation`；尚未执行 `failure return`，不声称该 gate 已通过。

- 当前工作树由 owner thread 依 `source_draw_index` 预排序，并在 `prepare_batch_plan` 内完成 variant id 解析、cache lookup 与统计归属；worker 仅消费不可变 `PreparedBatchPlan`，不持有 variant/cache mutex。
- worker 通过调用方传入的 `TaskPool::install` 并行生成独立 `PreparedBatchChunk`；`IndexedParallelIterator` 的收集顺序和 owner thread 的单点 merge 保持 graph/source 顺序，cache store 与统计也只在 owner thread 提交。
- 重复 cache key 会在进入 worker 前回退既有串行 owner 路径，避免同帧重复 identity 产生错误的并行 miss/store 事务。
- `render_perf_parallel_prepare_deterministic_sort` 连续覆盖 generation 1 miss/rebuild 与 generation 2 hit，逐元素比较 serial/TaskPool command signature，并比较完整 cache stats。
- 待证据：当前源码 focused mesh command tests、原始 reproduction 与 Render17 PF-M2 runtime gate。只有这些 managed Cargo 结果完成后才可改为 `fixed` 并回传来源计划。
