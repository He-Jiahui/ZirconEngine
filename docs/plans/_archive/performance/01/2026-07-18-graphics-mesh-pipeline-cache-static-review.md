---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
  - dev/bevy/crates/bevy_render/src/texture/fallback_image.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshDrawCommands.cpp
tests:
  - mesh_pipeline_cache subtree 17 of 17 Rust files reviewed, 4499 baseline lines
  - seven ensure cache-hit ordering source guard RED then GREEN
  - rustfmt passed for eight touched Rust files
  - current-source Cargo, startup counters, F0/F2 and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics mesh_pipeline_cache整个模块逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`mesh_pipeline_cache/**`当前17/17个Rust文件、基线4,499行，覆盖cache构造、7类pass pipeline ensure、variant registry、WGSL source assembly、disk cache、prewarm WGPU validation、forward binding和全部测试。MVP热路径有三层成本：每batch/pass先构造宽owned variant key（PERF-MVP-355）；首次miss同步模板拼装、全文hash、disk lookup/write和driver pipeline create（PERF-MVP-356）；cache构造又无条件实例化多类optional feature GPU资源（PERF-MVP-390）。

## 已直接止损

Base、OIT、GBuffer、depth prepass、shadow、velocity、TAA reactive/material mask 7条公开ensure原来即使pipeline map已命中，也先从registry clone `PipelineKey`/`ShaderVariantKey`，clone geometry descriptor，组装完整WGSL并hash/格式化module key，最后才查pipeline map。现在每条路径在variant投影前直接返回对应cached pipeline；TAA以只读helper覆盖两张map。统一源码门禁先RED后GREEN，`rustfmt`通过。

这项修改不改变miss、hot reload或device重建语义：variant id已经包含pipeline/material/geometry/quality revision；同一cache内存在pipeline即代表该id的source已成功实例化。后续真正的source generation变化必须产生新variant id，而不是让稳定id每帧重新探测source。

## 剩余P0瓶颈

- PERF-MVP-355：registry命中前仍完整构造并hash owned `MeshPipelineVariantKey`，miss report又clone宽诊断；应把dense variant id编入material/static generation artifact。
- PERF-MVP-356：首次miss仍在render submission线程组装/全文hash WGSL，执行disk metadata/JSON/zstd读写并同步`create_shader_module`/`create_render_pipeline`。应由Render08 queued pipeline state与worker/driver lane接管，render只O(1)读取Ready/Loading/Error。
- PERF-MVP-368：forward shading每pass仍创建volumetric params buffer、拼接约数十项entry Vec并创建bind group；继续由Render02 generation bundle/ring处理。
- PERF-MVP-390：`MeshPipelineCache::new`无条件构造transmission、light-cookie、irradiance-volume、reflection-probe、lightmap、volumetric与OIT fallback/owner。Render02/11/18须拆成per-device共享neutral与feature-generation懒建owner，minimal F2不能为disabled optional features创建真实资源。

prewarm validation也会逐request创建shader module/render pipeline并同步`pollster::block_on(error_scope.pop())`；它属于PERF-MVP-357的离线/有界worker验证，不在frame线程修补。参考Bevy queued `PipelineCache`的异步状态与共享`FallbackImage`生命周期，不复制其API。

## 验收

按variants 1/100/10k、passes 1/7、cold/warm/corrupt disk、minimal/all features、renderer owners 1/8记录variant/key/source/hash bytes、disk I/O、module/pipeline/texture/buffer/sampler/layout creates、queue writes、driver stall、RSS与CPU p50/p95/p99。当前目标：stable ensured pipeline的variant projection/source assembly/hash/module key为0；最终frame同步disk/driver work=0，minimal optional real-resource create=0，共享neutral每device每kind≤1。Cargo、F0/F2、hot reload、像素、timestamp与DX12 RenderDoc资源/事件序通过后才能移入`review.md`。
