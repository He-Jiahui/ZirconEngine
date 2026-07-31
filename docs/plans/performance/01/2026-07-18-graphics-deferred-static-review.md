---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/05-lighting-shadows.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ClusteredDeferredShadingPass.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/DeferredShadingRenderer.cpp
  - dev/bevy/crates/bevy_pbr/src/deferred/mod.rs
  - dev/bevy/crates/bevy_pbr/src/render/mesh_view_bindings.rs
tests:
  - deferred subtree 17 of 17 Rust files reviewed, 1761 current lines
  - paired pipeline foundation and fixed attachment source guards RED then GREEN
  - rustfmt and scoped diff checks passed
  - current-source Cargo, scale counters, F2 Forward+/Deferred pixels, timestamp and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics scene_renderer/deferred整个模块逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/deferred/**`当前17/17个Rust文件、1,761行，覆盖GBuffer格式、scene resources、geometry replay、lighting layout/pipeline/source assembly及全部Rust tests。MVP Deferred的主要CPU热点是稳定帧重复构造binding artifact；启动热点是两个lighting variants重复准备相同shader foundation和未使用feature资源。

## 已直接止损

- `DeferredSceneResources::new`原来为普通与SSS MRT pipeline分别导出shading-model includes、构建source request、解析module graph、拼接WGSL、创建shader module和相同pipeline layout。现在通过paired constructor共享一次foundation，只保留两个必要render pipeline objects；source/module/layout 2→1。
- lighting pass的1或3个color attachments原来每frame/camera创建Vec，现改为固定3槽栈数组和active slice。pipeline target列表也改固定数组，删除构造期小Vec。两组源码门禁先RED后GREEN，新增Rust source guards，`rustfmt --check`与scoped diff通过。

## P0瓶颈与路由

- PERF-MVP-368：`execute_lighting`每frame/camera创建volumetric params buffer，物化GBuffer/shadow/grid/environment/lightmap/volumetric/cookie/irradiance的20+ binding entries并创建大bind group；GBuffer录制也重建forward shadow receiver group。`write_scene_uniform`还稳定帧clone四个lightmap Arc后覆盖Deferred owner。Render02/05/18须按resource generation发布唯一binding bundle与dynamic uniform offsets，stable create/clone/entry allocation=0。
- PERF-MVP-390：Deferred构造独立创建shadow compare sampler、1×1 atlas view、slot/global fallback buffers和volumetric fallback，与MeshPipelineCache/ShadowMapRenderer的neutral owner重复；无SSS材质时仍创建SSS MRT pipeline。Render05/08/11/18须共享per-device neutral并按compiled feature generation lazy single-flight。
- PERF-MVP-356：两条render pipeline descriptor仍为`cache: None`且在renderer构造线程同步driver compile。paired foundation只消除了重复前端准备，未冒充跨启动driver cache或异步pipeline完成。

参考UE clustered deferred pass的view/pass parameter owner和按feature permutation调度；Bevy仅用于对照prepared view bindings与deferred pipeline specialization。保留现有GBuffer ABI、SSS MRT、custom shading-model include、shadow/cookie/volumetric/irradiance绑定和Forward+/Deferred像素等价。

## 验收

按Deferred off/on、SSS off/on、cameras 1/8、passes 1/8、stable/1% resource change、cold/warm process记录shader source/module/layout/pipeline creates、driver wall、params buffer/bind-group creates、entry/attachment alloc、lightmap handle clones、upload与CPU/GPU p50/p95/p99。当前source/module/layout=1、attachment/target Vec=0；最终stable lighting/GBuffer binding create、entry alloc和lightmap clone=0，bundle≤1/resource tuple generation，无SSS时SSS pipeline=0，共享neutral每device每kind≤1。current-source Cargo、custom shading model/WGSL、F2 Forward+/Deferred/SSS像素、timestamp与DX12 RenderDoc通过前留在`pending.md`。
