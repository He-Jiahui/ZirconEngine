---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/cached_mesh_draw_commands.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshDrawCommands.cpp
tests:
  - pending command cache extract eleven of eleven Rust files reviewed, 1471 current lines
  - full-hit, lazy rebuild, residual fallback, visibility and second-frame tests reviewed
  - production root currently leased by active Render02 owner; no overlapping edit attempted
  - current-source focused Cargo and F2 counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics pending mesh command cache extract逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`pending_command_cache_extract.rs`与子目录当前11/11个Rust文件、1,471行，包括full-hit、partial miss、lazy non-material rebuild、residual fallback、visibility prune和second-frame测试。该层能在material-bound phase缺失时延迟到完整MeshDraw构建，residual draw Vec也只在首次残留时分配；这些设计避免了更大的无效构建。剩余稳定命中分配归PERF-MVP-382。

## PERF-MVP-382补充：full hit仍有per-draw容器与command clone

每个eligible pending draw先调用`cacheable_phases_for_extract_item`创建capacity 3的phase Vec。即使三phase 100% cache hit，`commands_for_extract_item_with_stats_and_context`仍为该draw再创建capacity 3的commands Vec，`lookup_status`逐phase clone整份owned `MeshDrawCommand`，外层随后把这些commands逐个push到frame总表。也就是说“跳过MeshDraw构造”没有等于“零per-draw allocation/clone”。visibility全裁剪仍创建空commands Vec，但phase Vec已经发生。

Render02应把最多3个cacheable phase表达为固定栈mask/array，并让hit直接把generation-owned shared command handle或最终phase range写入唯一frame arena；不要先创建per-draw commands Vec。cache miss的lazy batch重建保持，但输出也直接落phase slot。该production root由活动`render02-md-m2-pending-draw-move-partition-20260717`租约保护，本会话只记录证据与计划回流，没有覆盖对方源码。

## 验收

按draws 0/1/1k/100k、phases 0/1/3、cache hit 0/50/100%、visibility visible/pruned、residual 0/1/50%记录phase/command/residual Vec alloc+capacity、cached command clone bytes、GPU handle clone、moved commands、cache probes与CPU p50/p95/p99。最终100% stable hit per-draw heap alloc=0、resource/command clone=0，visibility-pruned alloc=0；miss只为实际rebuild分配且changed generation≤1。现有full-hit/lazy/fallback/visibility/second-frame语义、Cargo和F2通过前留在`pending.md`。
