---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/history
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/04-visibility-culling.md
  - docs/plans/zircon_runtime/render/06-temporal-pipeline.md
  - docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneViewState.cpp
  - dev/bevy/crates/bevy_render/src/texture/texture_cache.rs
tests:
  - history baseline 9 of 9 Rust files reviewed, 368 baseline lines
  - obsolete clear_texture two-file module deleted, current subtree 7 files and 416 lines
  - GPU attachment initialization source guard RED then GREEN
  - rustfmt and scoped diff checks passed
  - current-source Cargo reservation 461d79d7bbe7445eb9645f3e8bfb7509 pending behind FIFO
  - scale counters, F2 history/cut/resize pixels, timestamp and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics scene_renderer/history整个模块逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/history/**`基线9/9个Rust文件、368行；直接修复删除obsolete `clear_texture`两文件后，当前目录7个Rust文件、416行。覆盖TAA双缓冲、GI/metadata、AO、SSR、HZB、exposure、volumetric history构造/有效位/flip与resize匹配。MVP最大问题是CPU整图初始化和feature无关的巨型资源整包所有权。

## 已直接止损

原构造对TAA read/write、GI、GI metadata、SSR分别分配RGBA16 CPU Vec并整图`queue.write_texture`，AO另分配RGBA8白图。3840×2160合计364,953,600 bytes（约365 MB/348 MiB）CPU payload与queue upload，每个history handle首次创建或resize都重做。

现在为六张纹理增加render-attachment usage，在同一command encoder中按attachment bytes/sample预算拆成4张HDR和SSR+AO两次无draw clear pass；黑/零confidence与AO白色语义不变。CPU full-texture allocation/upload降为0，queue submit一次；obsolete generic CPU clear模块已删除。源码门禁先RED后GREEN，新增Rust source guard，`rustfmt --check`与scoped diff通过。

## P0瓶颈

PERF-MVP-395：`prepare_history_textures`只在“是否需要任一history”处做总gate；进入后仍构造TAA双纹理、GI/metadata、AO、SSR、HZB和exposure全集。target size、HZB size/mips、TAA key或volumetric quality任一变化都会替换整包。`bind_history_graph_resources`虽按flags绑定，却每frame clone TextureView/Buffer handles进execution resources。

Render01须发布按feature mask与slot generation分拆的history binding plan；Render04/06/07/18分别维护HZB/TAA/post/advanced slot的size/quality revision，只有changed slot重建并精确失效。feature-off使用compiled graph不声明或共享neutral，不创建真实full-resolution history；stable binding引用dense handles而非重复clone。

参考UE view-state per-feature history与Bevy texture cache的descriptor-keyed reuse；保留camera cut invalidation、TAA/exposure flip、GI/volumetric有效位、HZB全mip copy和resize像素语义。

## 验收

按features逐项/all、history handles 1/8、720p/1080p/4K、stable/resize/render-scale/HZB-plan/froxel-quality change记录texture/view/buffer create/destroy、VRAM、CPU init alloc/upload、GPU clear pass、handle clones、copy bytes与CPU/GPU p50/p95/p99。当前CPU init bytes=0、full pack clear passes≤2；最终feature-off slot=0、HZB-only不创建TAA/GI/AO/SSR、stable rebuild/handle clone=0、changed只重建affected slot。current-source Cargo、F2 TAA/GI/AO/SSR/HZB/exposure/volumetric cut+resize逐像素、timestamp与DX12 RenderDoc通过前留在`pending.md`。
