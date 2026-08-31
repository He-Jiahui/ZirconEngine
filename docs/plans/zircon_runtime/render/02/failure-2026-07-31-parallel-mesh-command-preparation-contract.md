---
handoff_kind: failure
status: open
created_at: 2026-07-31
summary_slug: parallel-mesh-command-preparation-contract
origin_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
fixing_plan: docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
origin_child_dir: docs/plans/zircon_runtime/render/17
fixing_child_dir: docs/plans/zircon_runtime/render/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
tests:
  - managed current-source zircon_runtime mesh command tests
  - render_perf_parallel_prepare_deterministic_sort
  - cached_static_command_hit_reprojects_current_batch_in_serial_path
  - cached_static_command_hit_reprojects_current_batch_in_parallel_path
  - render_parallel_prepare_normalizes_source_order_before_owner_transactions
  - render_parallel_prepare_duplicate_cache_keys_falls_back_to_serial_owner_path
  - render_parallel_prepare_predicate_requires_multiple_workers_and_batches
  - cached_prepare_profile_stages_preserve_owner_boundaries
  - cached_prepare_profile_counters_remain_single_observation_points
  - dispatch_reason_profile_codes_remain_stable
  - parallel_admission_uses_the_transaction_shader_quality
  - cached_parallel_worker_does_not_create_per_batch_timeline_spans
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
- worker 通过调用方 `TaskPool` 暴露的 `ParallelSliceExecutor::parallel_map_ordered` 生成独立 `PreparedBatchChunk`；有序结果和 owner thread 的单点 merge 保持 graph/source 顺序，cache store 与统计也只在 owner thread 提交，渲染模块不再直接依赖 rayon。
- 重复 cache key 会在进入 worker 前回退既有串行 owner 路径，避免同帧重复 identity 产生错误的并行 miss/store 事务。
- `render_perf_parallel_prepare_deterministic_sort` 连续覆盖 generation 1 miss/rebuild 与 generation 2 hit，逐元素比较 serial/TaskPool command signature，并比较完整 cache stats。
- cache hit 不再直接提交上一帧的完整命令：serial 与 TaskPool chunk 路径都保留缓存的 phase、pipeline kind 与 variant id，再从当前 `MeshBatchRef` 投影可见命令。`cached_static_command_hit_reprojects_current_batch_in_serial_path` 与 `cached_static_command_hit_reprojects_current_batch_in_parallel_path` 分别锁定 generation 2 命中、零 rebuild 时更新 sort key、source draw index、GPUScene instance span 与 direct first-instance 参数。
- `render_parallel_prepare_normalizes_source_order_before_owner_transactions` 以逆序且 cache identity 不同的 batch 输入重复 generation 1/2，对照 serial 与 parallel 的 command signature 和完整 cache stats，锁定 owner transaction 在 source-order normalization 后发生。
- `render_parallel_prepare_duplicate_cache_keys_falls_back_to_serial_owner_path` 以两个 source draw index 不同、但稳定 cache identity 相同的静态 batch 重复 generation 1/2，直接断言 `should_prepare_batches_in_parallel` 为 false，并对照 serial 与 parallel 的 command signature 和完整 cache stats，锁定重复 key 必须绕过 worker 路径。
- `render_parallel_prepare_predicate_requires_multiple_workers_and_batches` 直接覆盖 single-worker、single-batch 与可并行的双 batch 输入，锁定并行调度只在具备实际 worker 并行度且工作量至少为两个 batch 时启用。
- `ParallelPreparationMode` 固化 parallel、single-worker、small-batch、duplicate-key 四种准入结果和稳定 profile code；串行/降级/并行路径共用 cache hit/miss、rebuild、command count 结果 owner，并分别以同名 `seal_phase_buffers` span 隔离 phase partition/sort，worker 内不创建逐 batch timeline span。
- dispatch 元数据与完成结果分别通过一次 `record_counter_batch` 发布，profiling capture 下每组只获取一次 recorder lock；普通非 profiling 构建不会构造计数数组。
- 已有序的产品串行入口继续直接进入 ordered helper；generic serial 与 parallel dispatcher 共用唯一 `normalize_source_order` owner。串行内部以 `serial_prepare_and_project` 区分 transaction+projection，parallel 以 `parallel_admission` 隔离准入检查和 duplicate-key 扫描；该扫描使用与后续 cache transaction 相同的 shader-quality key 维度，再进入 owner transaction -> worker projection/wait -> ordered merge -> seal。
- 源码契约测试锁定产品串行零排序快路、通用 normalization 单一归属、阶段顺序、dispatch/result counter 单一归属、串并行结果模式对称性、原因码和 profile span 成本边界；这些检查只证明观测契约，不替代 WPR/xperf 样本或性能结论。
- 本轮 scoped `rustfmt --check`、源码契约和 `git diff --check` 已通过；既有受管 mesh gate receipt 记录过 Cargo 启动前的共享 target 未托管 artifact 拒绝，本次最终源码快照尚无精确 managed compile ticket，未使用 raw Cargo 绕过协调器。
- 待证据：当前源码 focused mesh command tests、原始 reproduction 与 Render17 PF-M2 runtime gate。只有这些 managed Cargo 结果完成后才可改为 `fixed` 并回传来源计划。
