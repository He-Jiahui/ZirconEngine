---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/ShaderPipelineCache.cpp
tests:
  - mesh_pipeline subtree 10 of 10 Rust files reviewed, 1766 current lines
  - descriptor and fallback shader tests inspected
  - current-source Cargo, cold-warm driver counters, F0/F2 and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics mesh_pipeline整个模块逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`mesh_pipeline/**`当前10/10个Rust文件、1,766行，覆盖Base/OIT/GBuffer/depth/shadow/velocity/TAA pipeline descriptors、fallback WGSL拼接和测试layout。生产代码没有新的逐帧CPU分配根因：descriptor只在`MeshPipelineCache` miss时创建，vertex/target arrays为固定临时数组，fallback shader以`concat!(include_str!(...))`在编译期形成静态`&str`。

## 剩余性能责任

7类`wgpu::RenderPipelineDescriptor`都设置`cache: None`。现有`ShaderVariantCacheDisk`只缓存WGSL及其压缩/元数据，不能证明跨进程复用driver binary pipeline；因此cold start仍会同步承受`create_shader_module`/`create_render_pipeline` driver compile。该根因已由PERF-MVP-356覆盖：Render08的Queued→Ready状态机除source/disk worker外，还须管理device/adapter/driver兼容的pipeline cache artifact或后台driver lane，失配时安全重编译，render submission线程不得阻塞。

shadow descriptor当前使用固定back-face cull而不是`PipelineKey.double_sided`，属于渲染语义而非本轮性能优化，不能借性能任务顺手改动。大量test-only layout/GpuScene创建不进入产品热路径；prewarm规模问题已归PERF-MVP-357。

## 验收

按cold/warm process、variants 1/100/10k、passes 1/7、adapter/driver stable/changed记录module/pipeline creates、driver compile wall、pipeline cache hit/miss/bytes、frame stall和RSS。最终warm compatible cache的frame-thread driver compile为0；cache失配/损坏走有界重建且产物等价。Cargo、F0/F2像素、hot reload、timestamp与DX12 RenderDoc事件序通过前留在`pending.md`。
