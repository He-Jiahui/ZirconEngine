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
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder/parallel_admission.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder/parallel_preparation.rs
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
  - parallel_preparation_has_one_folder_backed_owner
  - mesh_pass_materialization_profile_counts_partition_and_unconditional_bucket_sorts
  - mesh_pass_materialization_profile_carries_producer_staging_merge_cost
  - mesh_pass_materialization_profile_carries_producer_staging_arena_cost
  - mesh_pass_materialization_profile_carries_discarded_pre_mesh_attempt_cost
  - pending_command_cache_extract_preserves_hit_cost_before_residual_fallback
  - mesh_pass_materialization_profile_accumulates_prebuilt_residual_merge_cost
  - mesh_pass_materialization_profile_counts_half_resolution_fallback_merge
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
- 并行 setup 已硬切到具名 `builder/parallel_preparation.rs` owner；`parallel_admission.rs` 仅保留准入策略，`builder.rs` 仅保留公开入口、串行编排和共享 cache 操作。按物理行计数，生产 owner 从单个 782 行 root 收敛为 459 行 root + 336 行 parallel owner，均低于结构规范 800 行 review warning；root 不保留兼容实现、worker DTO 或第二套调度状态。
- profiling 源码契约已同步到拆分后的物理 owner：串行边界在 `builder.rs` 内闭合，并行边界只读取 `parallel_preparation.rs`，不再用已迁移符号作为 root 结束标记而在测试运行时 panic。
- Unreal 5.5 复审确认当前 blocking `parallel_map_ordered` 仍不是 `FParallelMeshDrawCommandPass` 的异步 setup-ticket 设计；每 batch plan/chunk Vec 与 seal 时十个 phase Vec 是下一轮必须先用 allocation stack/WPR 定位的结构风险。详细硬切不变量已写入 performance plan；在产品 profile 前不实施也不宣称该优化完成。
- 规模场景基础设施复审确认 `New-RenderExtractScaleProject.ps1` 可生成 1..100,000 个独立静态实体，当前 collect/GPUScene 路径保持每稳定实例 `instance_count = 1`；但 primitive/entity 数不等于多 pass command 数，验收必须同时读取 `mesh_commands.batch_count` 与 `mesh_commands.command_count`。
- `RenderExtractMeshCommandMetrics.psm1` 已接管 `mesh_command_preparation` 与 `mesh_command_parallel_dispatch` 两个显式覆盖记录；缺少任一必需 stage/result/dispatch/结构成本 counter、使用非 `render` stream，或任一 attempt 证据不完整时均 fail-closed。准备覆盖还会依据直接串行、调度降级、并行三种实际分支分别要求对应 processor span；同一 attempt 混合降级与并行时要求两边并集，并支持 aggregate `statistics.min/max`，避免只有外层 stage 就误判为 measured。report owner 保持 796 行，新模块 309 行。旧路径 recorder 在 profiling build 发布 26 个 source-bound 结构计数，覆盖 final/producer command arena grow/peak、build/partition/merge/finalize/sort、visit 数、十个 bucket 长度、accepted/aborted cache payload `Arc` clone 与 command/view generation 零复用基线，并补齐 extract、pre-Mesh materialize、old-path finalize、indirect plan、replay record stage。普通 build 不保留统计字段或逐 push 容量检查。focused suite 和最新全量复跑均为 18/18 通过。这只关闭观测报告实现，不替代 WPR/xperf、RenderDoc、产品截图或 managed Rust 编译，也不把 fixture 通过记为产品验收。
- Render02 static-cache follow-up: `MeshPipelineResolverConfigurationEpoch` now gives resolver policy changes a typed, source-owned invalidation dimension. The registry advances it only on real environment-profile transitions; serial, TaskPool parallel, and pre-MeshDraw cache boundaries synchronize once, clear stale payloads/pipeline pins, and publish `cache_invalidated_resolver_configuration_count` through the existing mesh queue stats and diagnostic store. The former environment-only provider callers no longer clear the cache directly. This closes the researched resolver-configuration contract without changing view-local cache keys or claiming arena migration/performance improvement.
- WPR 观测基础设施现已把 CPU 与 Heap 设为互斥的独立产品运行：`-UseWpr` 导出 PID/生命周期限定的 sampled stacks，`-UseWprHeap` 导出按 total allocation 排序的分配栈；ETL、xperf 文本和 JSON 回执全部留在 D/E/F 的 invocation trace 根。xperf 直接消费产品 `StartTime`/`ExitTime` 的 UTC wall-clock，不再用 WPR 调用时间估算 ETL 相对起点。报告会验证 schema/evidence kind、profile、PID、产品/运行时间窗、xperf range、字节数与 ETL/analysis SHA-256，legacy raw ETL 不能提升 measured 状态，Heap run 的内部时间线标为 `instrumented_not_baseline`。当前模块契约 6/6、capture 27/27、evidence 10/10、publication 12/12、metrics 11/11 通过。尚未执行 current-source 产品 WPR，因此没有性能、分配或功耗结论。
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
- 本轮 scoped `rustfmt --check`、源码契约和 `git diff --check` 已通过；默认 feature build + `mesh_pass_materialization_profile` focused lib tests 以及 profiling feature build 的 dry-run 命令均绑定到 `E:\cargo-targets` 协调器池。首次实际受管请求 `44fd401668874dd690e34a83a8eeb9d8` 在 Cargo 启动前以 `cargo_reuse_pool_busy` 终止，冲突 job 为 `5ce97a748a48486fa20bb72b1ba3dd3f`；未重复提交、未使用 raw Cargo 或备用 target 绕过协调器，本次最终源码快照仍无精确 managed compile ticket。
- 待证据：当前源码 focused mesh command tests、原始 reproduction 与 Render17 PF-M2 runtime gate。只有这些 managed Cargo 结果完成后才可改为 `fixed` 并回传来源计划。

## 2026-09-19 受管静态合同回执

协调器票据 `aed8e1cb201c4057a2a3d368114d84c6` 已在 Windows-native
静态合同执行中通过（job `9cff7ff7479d40138ece8ec75e9c7795`，run
`aed8e1cb201c4057a2a3d368114d84c6`，exit code `0`）。终端输出为
`RENDER02_PARALLEL_MESH_PREPARATION_SOURCE_CONTRACT_PASS`，检查 7 个受管
路径，并明确保留 Render11 所有的
`scene_renderer_core_render_compiled_scene/render/render.rs` 重叠，不把它
计入本生命周期。当前源码 manifest 为
`9ca922798471ca0dbc66a153e4c473b22ea554ad20a6d82bf0f67497259a9695`。

该回执只证明静态 owner/cache/profiling 合同；受管 Windows Cargo focused
tests、原始 reproduction、Render17 PF-M2、WPR/xperf、WGPU/RenderDoc、独立
C/I/M 审查、canonical fixed return、closeout SHA 与 WeCom 结果仍待完成。
外部 `E:\\Git\\zr_vm` 脏工作树继续阻止新的 Cargo 票据，failure 保持 open。

## 2026-09-20 independent source review r2

The independent reviewer inspected the current Render02 owner boundary and the
direct producer/consumer path without editing production sources.  The review
covered the immutable `PreparedBatchPlan` and ordered `TaskPool` projection,
duplicate-cache-key admission, serial/parallel cache-key and shader-quality
parity, current-batch cache-hit projection, phase partition/seal ordering,
profiling counter ownership, and the resolver configuration epoch.  The
Render11-owned `scene_renderer_core_render_compiled_scene/render/render.rs`
overlap remains explicitly deferred and was not absorbed.

Read-only checks:

- `rustfmt +1.94.1 --edition 2021 --config skip_children=true --check` over
  the seven owned Rust paths: `RUSTFMT_PASS paths=7`.
- `git diff --check` over the same paths: pass (only Git line-ending notices).
- Existing managed static receipt `aed8e1cb201c4057a2a3d368114d84c6` remains
  the only accepted validation ticket for this snapshot (job
  `9cff7ff7479d40138ece8ec75e9c7795`, run equal to the ticket, exit code `0`,
  marker `RENDER02_PARALLEL_MESH_PREPARATION_SOURCE_CONTRACT_PASS`).

Current source hashes inspected by the reviewer:

```text
zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder.rs 3602d4c60c731dae775da7d04d024ebb0335fb83b6be77e0bcc2b49f142f3db9
zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder/parallel_admission.rs 99ea0bf88eb5b01223a8e4eba5edc62cdba70d3abc61731c935a5397b1d63fdb
zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder/parallel_preparation.rs 5058896e897982a581f83c26d4edd31a51f1c6aa840d4e0b5640286153b32b13
zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder/profiling_contract_tests.rs a778303eec7e9e105649b0019393c4acad39a5669ad4f97de0080fc79c60b0c1
zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/tests/cache.rs c50a6ea0e3641c2d86e27cebb128eb512d2208b24cfe2cdee8cf2a336f5e99a7
zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs 79530d1cb0581b995e071b77437520226220dda40e08a3e14bf03ba3c3473813
zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs 0084c993d8fb20dcfbfa370be0cb964ab698e2ff515ac94240b0d0775d3c5c31
```

Independent review result: `Critical=0 Important=0 Moderate=0`.  This receipt
does not claim managed Cargo, Render17 PF-M2, WPR/xperf, WGPU/RenderDoc, product
image, canonical return, or closeout; those gates remain open.
