---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/04-visibility-culling.md
  - docs/plans/zircon_runtime/render/05-lighting-shadows.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ShadowSetup.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ShadowDepthRendering.cpp
  - dev/bevy/crates/bevy_render/src/render_phase/mod.rs
tests:
  - shadow subtree 12 of 12 Rust files reviewed, 3509 baseline lines
  - fixed point-light face allocation source guard RED then GREEN
  - rustfmt passed for shadow plan and tests
  - current-source Cargo, scale counters, F2 shadow pixels, timestamp and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics scene_renderer/shadow整个模块逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/shadow/**`当前12/12个Rust文件、基线3,509行，覆盖CSM/punctual view projection、frame plan、atlas allocator/resources/bindings、GPU slot ABI、shadow command replay与全部测试。最小3D场景的主要瓶颈不在矩阵数学，而在三次重复物化：每帧全量allocator/plan、每slot独立GPU对象与全command扫描、每帧全量slot/global上传。

## 已直接止损

点光每个light固定需要6面allocation，原来通过iterator `collect::<Option<Vec<_>>>()`每灯分配heap Vec。现在用栈上`[Option<ShadowSlotAllocation>; 6]`，任一面缺失仍整灯跳过，完整时按0..5原顺序写连续slots。源码门禁先RED后GREEN，`rustfmt`通过。

## P0瓶颈

- PERF-MVP-391：录制对每slot创建scene uniform buffer、7-entry scene bind group和pass-name String，开启独立render pass；每slot又从visibility构造`BTreeSet<EntityId>`并扫描全部shadow commands。4 CSM加一个point light已经是10次全表扫描。Render05/04/02须交付单atlas pass、persistent uniform ring/dynamic offsets和per-view dense visible command ranges。
- PERF-MVP-392：allocator每frame重建dedup/planned/retained/free rects；preemption近retained×planned×challengers，free-rect每次reserve后全量O(F²)compact。plan又建allocation map、slots/passes/assignment containers并重算矩阵；resources无条件上传全部active slots与globals。Render05须按light/camera generation拆allocation、view matrix和upload dirty ranges，stable三者均为0。
- PERF-MVP-390补充：`ShadowMapRenderer::new`仅为full scene bind layout独立创建3个1×1 environment cubes、BRDF LUT、sampler和SH buffer，与renderer已有neutral环境资源重复。应共享per-device neutral owner或使用shadow所需的最小layout。

参考UE shadow setup/depth rendering的cached shadow state、view relevance与并行/批量录制边界；Bevy只作为render phase/dense view数据参考。保留现有atlas hysteresis、tier downgrade、cascade稳定和像素语义，不以简化功能换取数字。

## 验收

按lights 0/1/100/1k、slots 0/4/16/64/256、commands 1/1k/100k、visible 0/10/100%、stable/1% add/remove/priority/camera move记录container alloc/hash/sort/free-rect visits、matrix builds、BTreeSet/set probes、command visits、pass/buffer/bind-group/String creates、upload ranges/bytes及CPU/GPU p50/p95/p99。当前point-face temp Vec=0；最终stable allocator/plan/upload=0、warm per-slot对象/String/set=0、atlas pass≤1、command visits近visible commands。Cargo、F2 directional/point/spot/PCF像素、timestamp与DX12 RenderDoc通过前留在`pending.md`。
