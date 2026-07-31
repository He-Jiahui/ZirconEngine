---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_pbr/src/render/mesh.rs
  - dev/bevy/crates/bevy_pbr/src/render/skin.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshDrawCommands.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GPUSkinCache.cpp
tests:
  - scene_renderer mesh subtree 107 of 107 Rust files reviewed, 21122 current lines
  - component source guards and rustfmt checks passed
  - current-source Cargo, scale counters, F0/F2, timestamp and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics scene_renderer/mesh整个模块逐文件性能静态审查（2026-07-18）

## 模块验收记录

`scene_renderer/mesh/**`当前107/107个Rust文件、21,122行已逐文件静态读完。为避免总记录过长，详细证据按模块拆在同目录：`prepared_queue` 8/8、`mesh_draw` 13/13、`mesh_pass` 23/23、`build_mesh_draws` 33/33、`mesh_pipeline_cache` 17/17、`mesh_pipeline` 10/10；顶层`skinning` 2/2与root wiring 1/1由本记录收口。

直接止损包括candidate grouping单表无key clone、command冗余sort/per-draw Vec删除、single raster固定数组、phase input O(M²)→O(M)、7类pipeline稳定命中前置返回等。剩余MVP P0集中在PERF-MVP-381..390：诊断重扫、cache hit owned临时量、phase/indirect唯一artifact、material binding、morph/skin/VG generation资源、动态mesh resident、同步pipeline compile和optional feature owner。

## 顶层skinning补充

`SkinnedMeshJointPaletteStorage`固定携带256个mat4并先用IDENTITY初始化完整数组，`from_matrices`只覆盖active prefix；`create_buffer`/`write_buffer`仍上传整个约16 KiB结构。current+previous的1,000实例测试预算约32.8 MiB，即使常见64骨也付全块初始化、复制和上传。该事实已并入PERF-MVP-386：compiled skeleton与pose generation之外，Render03还须提供persistent palette slot/ring和active-prefix dirty upload，stable bytes=0、changed bytes近active bones；固定ABI可保留最大上限，但不能等同于固定传输量。

root `mod.rs`只做模块挂载与re-export，没有运行时循环、分配或锁。当前静态结论不等于动态验收：Windows Cargo reservation仍非FIFO头，F0/F2、规模allocation/GPU object counters、timestamp与DX12 RenderDoc尚未完成，因此整个mesh目录继续留在`pending.md`而不进入`review.md`。
