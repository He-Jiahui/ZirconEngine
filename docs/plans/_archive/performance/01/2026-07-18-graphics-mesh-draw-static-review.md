---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list/builder.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/render_phase/mod.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshDrawCommands.cpp
tests:
  - mesh draw thirteen of thirteen Rust files reviewed, 1113 current lines
  - command builder and cache caller path traced
  - current-source focused Cargo, F2 counters and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics mesh draw逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`mesh/mesh_draw/**`当前13/13个Rust文件、1,113行，包括owned draw、queue profile/batch key、sort input、texture identity、direct/indirect、skinning与Virtual Geometry execution投影。profile/geometry状态均为紧凑Copy数据和O(1)分支，没有发现算法级问题；主要瓶颈位于draw→batch→command的重复owned投影，编号PERF-MVP-382。VG execution DTO仅用于stats的重复构建已归PERF-MVP-381，indirect args buffer逐frame重分配归PERF-MVP-376，不重复编号。

## PERF-MVP-382：cache命中前仍逐draw克隆batch资源

`MeshDraw::mesh_pass_batch_ref`为每个draw构造owned `MeshBatchRef`，clone `PipelineKey`、mesh `Arc`、material与standard wgpu bind group、可选GPU-scene bind group、previous geometry `Arc`及indirect buffer `Arc`。两个command builder入口都先对全部draw执行该投影，随后才查询`CachedMeshDrawCommands`；因此稳定静态场景即使command cache 100%命中，仍发生全部资源handle原子增减和pipeline key owned clone。下游mesh-pass审查已把动态/cache-ineligible路径的小型per-draw command Vec改为直接写frame arena；cache miss按phase重建的临时Vec与batch owned投影仍待收敛。

Render02/03应使用borrowed `MeshBatchView<'frame>`或generation-owned dense prepared-batch handle；cache key/hit只读取identity/revision，命中时不取得任何owned GPU handle。只有实际重建command时才clone必须长期持有的资源。动态命令直接写入一次预留的frame arena，并通过range/count更新stats，禁止per-draw Vec。Bevy的binned phase以batch key/entity和extra index驱动准备，Unreal cached mesh draw command也先复用缓存命令；Zircon不照搬容器，但应保持cache hit前投影为轻量borrowed identity。

## 验收

按draws 0/1/1k/100k、static cache hit 0/50/100%、phases 1/6、direct/indirect记录batch projection数、Arc/wgpu/key clone、临时Vec alloc/grow、moved commands与CPU p50/p95/p99。当前dynamic/cache-ineligible per-draw temp Vec=0；最终stable 100% hit resource/key clone=0、所有路径per-draw temp Vec=0且frame command arena grow≤1；命令内容、phase过滤、sort、cache invalidation、Cargo、F2像素和DX12 RenderDoc一致前留在`pending.md`。
