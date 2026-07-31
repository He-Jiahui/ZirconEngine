---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/05-lighting-shadows.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LightGridInjection.cpp
  - dev/bevy/crates/bevy_pbr/src/cluster/gpu.rs
tests:
  - lighting subtree 4 of 4 Rust files reviewed, 1027 current lines
  - current-source Cargo, scale counters, F2 lighting pixels, timestamp and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics scene_renderer/lighting整个模块逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/lighting/**`当前4/4个Rust文件、1,027行，覆盖light packing、cookie/volumetric metadata、CPU clustered grid构建、GPU buffer写入及pass glue。最小3D场景的主要瓶颈不是单个数学操作，而是同一lighting generation在GPUScene与light-grid两个消费者中重复投影，并在每个view稳定帧重复分配、清零、统计扫描和全量上传。

## P0瓶颈

- PERF-MVP-393：`pack_lighting_extract_with_cookies`既在`build_mesh_draws`中为GPUScene lights执行，又在`build_light_grid_for_frame`中执行；两次都可重建cookie frame plan。advanced metadata还对每个packed light线性扫描volumetric light IDs，复杂度为O(lights×volumetric IDs)。Render05/03/18须发布按lighting/cookie/volumetric generation唯一的`PackedLightingFrameArtifact`与dense membership，两个消费者只引用同一产物。
- `build_light_grid`每次重新分配并清零zbins、tile masks和zbin min/max，然后按灯光覆盖tile/bin；构建结束又无条件调用`light_grid_stats`，再次执行bins×tiles×words的笛卡尔扫描。诊断关闭时这份工作没有产品价值，开启时也应融合主构建或只对sealed generation计算一次。
- `write_light_grid_buffers`对params、全部zbins和全部tile masks无条件全量写入。Render05须按camera+lighting generation复用CPU/GPU capacity或迁移到GPU compute，并只上传dirty/active ranges；stable generation的grid build与upload都应为0。

参考UE light-grid injection的GPU/并行构建边界与Bevy clustered-lighting GPU结构；不把局部`contains`换容器当成完成，因为重复pack、稳定帧全构建和诊断二次扫描才是共享根因。

## 验收

按lights 0/1/100/1k/65k、cameras 1/8、720p/1080p/4K、stable/1% changed、diagnostics off/on记录pack/cookie builds、volumetric membership probes、container alloc/zero bytes、tile/bin visits、stats cluster visits、upload calls/ranges/bytes及CPU/GPU p50/p95/p99。最终要求packed artifact build≤1/lighting generation，stable view/light generation的grid build/upload=0，diagnostics off时Cartesian stats visits=0，changed工作近affected views/lights。current-source Cargo、F2 Forward+/Deferred lighting像素、timestamp与DX12 RenderDoc通过前留在`pending.md`。
