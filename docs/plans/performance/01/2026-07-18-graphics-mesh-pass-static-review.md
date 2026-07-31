---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_pass_batch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/render_phase/mod.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshDrawCommands.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshDrawCommandStats.cpp
tests:
  - mesh pass twenty-three of twenty-three Rust files reviewed, 5134 current lines
  - pre-partition redundant sort source guard RED then GREEN
  - shared dynamic command arena source guard RED then GREEN
  - existing phase ordering, cache, processor, indirect, compaction and replay behavior tests retained
  - rustfmt and scoped git diff check passed
  - current-source focused Cargo, F2 counters and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics mesh pass逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`mesh/mesh_pass/**`当前23/23个Rust文件、5,134行，包括command/cache/build/processors、phase lists、indirect batch/compaction/execution/readback与replay，以及全部测试。replay用固定slot/pipeline/geometry状态跳过重复bind，command phase过滤与sort key为紧凑比较；这些路径合理。本切片确认owned batch/cache投影（PERF-MVP-382）和command/indirect artifact多次物化（PERF-MVP-383）两组MVP问题；同步readback和per-frame indirect GPU buffers已由PERF-MVP-373/376覆盖。

## 已直接止损

- 两个command builder原在全部processor输出后先`commands.sort()`，随后`MeshPassCommandBuffers::from_command_list`分到9张phase Vec并对每张再次sort。全局sort不改变每phase内部最终排序结果，现已删除，只保留最终phase sort；源码门禁RED→GREEN。
- uncached与cache-ineligible路径原每个batch创建`MeshDrawCommandList`小Vec，六个processor写入后再逐command搬入frame总表；static cacheable batch的TAA reactive mask也单独创建小Vec。现在直接写共享frame command list，并按写入前后长度更新rebuild/dynamic stats。cache miss需要筛选并存入phase cache的临时list仍保留；源码门禁RED→GREEN。

现有phase/order/count/cache tests锁定输出语义，rustfmt/scoped diff check通过；Cargo预约仍未到FIFO队首，不把源码门禁记为动态验收。

## PERF-MVP-382：cache hit之前的owned batch投影

`mesh_pass_batch_ref`在cache lookup之前clone pipeline和多类GPU handle；`CachedMeshDrawCommands::lookup_status`命中后还clone整份owned command。Render02/03须把cache identity/revision lookup前置到borrowed/dense batch view，并让cached command使用generation-owned shared handle；100% static hit不应发生batch resource clone。`retain_generation`当前每frame全扫cache entries，也应由scene removal/dirty generation或分桶sweep替代，避免稳定N-entry cache另做O(N)维护。

## PERF-MVP-383：command与indirect计划存在双重权威构建

`from_command_list`每build新建9张phase Vec并搬移所有commands；camera-stack `extend`对每张列表extend后再全量sort。`stats_with_indirect_batches`先逐phase扫描command stats，再为8组phase及transmission完整调用`IndirectDrawBatcher::build`；actual `MeshPassIndirectDrawExecutions::build`随后对相同commands再做一次batch key、PipelineKey clone、args Vec、batch Vec和compaction metadata构建并创建GPU buffers。stats不应成为第二个batch authority。

Render02/03应发布单一phase-owned command arena与range table，构建时直接写最终phase；camera stack对已排序range做linear merge或按generation复用。indirect batch/args/compaction plan每command generation只生成一次，同时供execution和sealed stats，GPU capacity复用继续由PERF-MVP-376交付。Bevy binned phase把batch sets/extra index作为prepare与render共享权威；Unreal也把visible mesh draw command收集与可选stats区分，Zircon应保持同类唯一artifact边界。

## 验收

按commands 0/1/1k/100k、phases 1/9、camera stack 1/8、static hit 0/50/100%、stable/1% changed、GPU-driven off/on记录sort calls/comparisons、phase Vec alloc/grow、moved commands、batch projection/handle clone、cache retain visits、indirect batcher builds、key clone/args/metadata bytes与CPU p50/p95/p99。当前门禁要求全局pre-partition sort=0、dynamic/cache-ineligible per-draw command Vec=0；最终stable sort/partition/batcher/cache full-scan=0，100% hit resource clone=0，changed sort≤1/affected phase且batcher≤1/generation，stats extra build=0。Cargo、F2像素、indirect/compaction parity和DX12 RenderDoc通过前留在`pending.md`。
