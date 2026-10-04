---
related_code:
  - zircon_runtime/src/core/framework/render/environment
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_brdf_lut.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/environment_ibl_compile_options.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_runtime_dispatch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/participation.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
reference_sources:
  - dev/cmft/src/cmft/cubemapfilter.cpp
tests:
  - environment twenty-one of twenty-one Rust files reviewed
  - product frame IBL cache GPU-scene lightmap and asset decode callers traced
  - reflection probe top-two source guard RED to GREEN
  - rustfmt and scoped git diff check passed
  - environment focused Cargo reservation pending FIFO
  - scale counters F0/F2 traces and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime render environment逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`core/framework/render/environment/**`当前21/21个Rust文件、6,770行，包括production与tests，并追踪renderer构造、frame compile options、IBL cache/dispatch、GPU Scene lightmap与Hybrid GI调用链。环境合同大量数据已用`Arc<[T]>`共享，但确认四个MVP性能根因。CPU reflection-probe fallback原分配并排序全部候选只取前二；当前产品caller为0，本轮仍以RED→GREEN单遍top-two止损，排序语义保持weight降序、priority降序、probe id升序，不把该收益冒充产品帧数据。

## PERF-MVP-351：renderer构造同步计算约1,678万BRDF积分样本

`SceneEnvironmentBrdfLut::new`在调用线程执行`build_environment_brdf_lut(128, 1024)`，即16,384 texels×1,024 Hammersley/GGX积分，包含大量sqrt/trig，再编码RG16F并上传；每个SceneRenderer构造都重复。该LUT由固定算法版本和常量决定，不应占用client/editor启动关键线程。

Render11/17应发布版本化预计算RG16F工件（构建期生成、产品只校验hash并上传）或一次异步GPU compute/cache；同device/adapter共享唯一handle，禁止每renderer重复。开发时保留CPU generator作离线黄金校验，不在产品构造调用。

## PERF-MVP-352：source-cubemap稳定帧同步读盘并多次深复制IBL payload

`compile_options_with_environment_ibl_bake_request`在每帧source-cubemap路径调用`resolve_ibl_bake_artifact_runtime_dispatch`；cache hit会同步`fs::read`完整`.zribl`，decode把payload再次`to_vec`，dispatch又为candidate和resolved blob深clone。cache miss同帧在frame尾writeback前再次读取。PMREM+SH9+IEM payload可达多MB，这把文件I/O、decode和内存带宽放到提交线程，稳定帧也不归零。

Runtime04与Render11/17应按`IblBakeArtifactRequest` identity + cache generation发布内存resident `Arc` artifact/descriptor state，异步加载一次并原子发布；frame只读O(1)状态，只有request或cache generation变化才resolve。metadata probe与payload load分离，writeback完成直接发布resident blob，不回读验证；payload/readback/dispatch使用单一shared owner。

## PERF-MVP-353：lightmap slot每draw/mesh线性扫描

`LightmapConsumeContract::slot_for_instance`对公开`Vec<(u64, Slot)>`线性find。GPU Scene `instance_data_for_pending_draw`对每个static pending draw调用一次，Hybrid GI participation又对每mesh调用一次，形成meshes×slots probes；同一light-set generation的结果没有共享索引。

Render03/11/17应在lightmap contract验证/发布边界构建唯一immutable id→slot索引（dense/hash按id分布选择），GPU Scene、Hybrid GI与diagnostics共享handle；serde Vec只作为输入/导出，不作为热查询权威。generation不变时index build=0，更新时一次validate+build。

## PERF-MVP-354：production cubemap/PMREM CPU路径未接入并行入口

external source cubemap decode与staging使用serial `build_source_cubemap_from_source_mips`；现有`ParallelSliceExecutor`入口只在tests出现。serial PMREM按mip×face×texel×32..128 samples做CPU积分，source mip angular filter也重计算cubemap face/mip offset；即使parallel入口被调用，粒度只有每mip最多6个face task，不能充分调度大面纹理。

Runtime04/Render11应把import/bake放到asset compute pool或现有GPU IBL compute，按mip×face×tile work-stealing；预计算layout offsets、solid-angle/Hammersley tables，限定scratch峰值。cmft把radiance工作组织为mip×face task list，可由最多64 CPU threads和OpenCL共同消费并回收unfinished任务；Zircon不照搬OpenCL ABI，但应采用同类细粒度调度和GPU优先策略。

## 验收要求

PERF-MVP-351记录1/2/8 renderer构造的LUT CPU samples/time/alloc/upload：产品CPU integrate samples=0、同device LUT create/upload≤1、startup主线程LUT time=0。PERF-MVP-352按artifact 64KB/1/16MB及stable/changed/miss记录frame-thread fs read/stat、bytes、decode、deep clone与stall：stable全部=0，miss异步且同generation read≤1。PERF-MVP-353按meshes/slots 1/1k/100k记录probes/index builds：查询近O(M)、stable build=0。PERF-MVP-354按64/128/512/1024 source、Fast/Normal/High记录worker利用率、tiles、offset recompute、CPU/GPU p95与scratch：调用线程blocking=0、production serial entry=0。环境/IBL/lightmap像素、seam、SH9/PMREM、cache invalidation、Cargo、F0/F2和RenderDoc通过前，本模块留在`pending.md`。
