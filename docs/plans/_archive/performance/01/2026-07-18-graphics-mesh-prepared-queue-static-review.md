---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshDrawCommands.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshDrawCommandStats.cpp
  - dev/bevy/crates/bevy_render/src/render_phase/mod.rs
tests:
  - mesh prepared queue eight of eight Rust files reviewed, 1462 current lines
  - shared candidate-group table source guard RED then GREEN
  - existing static, dynamic, instancing, velocity, skinning, LOD, GPU-scene and virtual-geometry behavior tests retained
  - rustfmt and scoped git diff check passed
  - current-source focused Cargo and F2 counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics mesh prepared queue逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读mesh `prepared_queue.rs`及`prepared_queue/**`当前8/8个Rust文件、1,462行，包括4个production文件和4个测试文件，并追踪两个产品caller。`prepared_mesh_queue_stats_for_pending_draws`在mesh draw主构建后无条件调用；`virtual_geometry_execution_stats`在compiled-scene draw输出上无条件调用。两者仅生产`PreparedMeshQueueStats`诊断，却在每frame/camera重复遍历和分配，归PERF-MVP-381。

## 已直接止损

`summarize_prepared_mesh_queue_items`原为static batch、dynamic batch和GPU instancing分别维护一张`HashMap<K, usize>`；同一eligible draw最多hash/entry三次，并在前两张表clone宽batch key。现在以单张`HashMap<K, CandidateGroupCounts>`同时累计三类计数，每个eligible draw只move key并做一次entry，key clone归零。输出仍分别统计size>1的group/draw，已有static/dynamic/instancing混合key测试覆盖原语义。源码门禁先观察RED再GREEN，rustfmt/scoped diff check通过；未把剩余第二遍诊断扫描冒充为解决。

## PERF-MVP-381：mesh诊断重复主构建工作

pending-draw stats在主构建结束后再次遍历全部draw，重新判断GPU skinning/geometry/phase/eligibility，为velocity状态再查一次`gpu_scene_entries`，并构造包含mesh、6类material binding/override、pipeline、pass flags和index range的batch key。单表止损后仍是O(draws)额外key/hash工作。VG stats随后把每个draw投影为完整`RenderVirtualGeometryExecutionDraw` DTO，再以segment-key与page两张`HashSet`计算unique/repeated数量；这些结果只进入last-frame stats。

Render02/03应把基础计数、candidate group和VG segment/page identity融合进命令/indirect plan的唯一generation artifact，Render17只读取sealed counters；stable generation不重建。若unique明细不能低成本随主循环维护，则只在显式diagnostics开启时构造。Unreal的mesh draw command stats由`r.MeshDrawCommands.Stats`控制，关闭时不调用pass draw-data收集；Bevy的batch准备直接产出phase batch identity，也没有为相同batch再维护三份key表。Zircon应保留always-on低成本总量，昂贵unique分析必须可关闭。

## 验收

用draws 0/1/1k/100k、unique keys/pages 1/1k/100k、stable/1% changed、diagnostics off/on记录draw/key/GPU-entry visits、hash/entry probes、key clone bytes、execution DTO、HashSet alloc/growth与CPU p50/p95/p99。当前门禁要求每eligible draw keyed entry≤1且key clone=0；最终stable extra scan/key/DTO/set=0，diagnostics off unique work=0，changed stats build≤1/generation，全部现有phase/batch/velocity/skinning/LOD/GPU-scene/VG计数等价。focused Cargo与F2产品trace通过前留在`pending.md`。
