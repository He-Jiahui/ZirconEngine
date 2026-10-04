---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/transparent
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
  - docs/plans/zircon_runtime/render/14-2d-stack.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/TranslucentRendering.cpp
  - dev/bevy/crates/bevy_core_pipeline/src/core_3d/main_transparent_pass_3d_node.rs
  - dev/bevy/crates/bevy_render/src/render_phase/mod.rs
tests:
  - transparent subtree 2 of 2 Rust files reviewed, 190 current lines
  - preallocation and in-place sort source guard RED then GREEN
  - rustfmt and scoped diff checks passed
  - current-source Cargo, scale counters, F2 transparent ordering/pixels, timestamp and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics scene_renderer/transparent整个模块逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/transparent/**`当前2/2个Rust文件、190行，覆盖mixed mesh/sprite submission projection、ordering tie-break与tests。该模块无GPU资源owner；MVP热点是每camera/frame重复扫描、物化和排序透明mesh/sprite索引。

## 已直接止损

`build_transparent_submission_order`原以空Vec开始扩展mesh与sprite，再用stable `sort_by_key`为完整数组分配/使用临时排序存储。现在先保留sprite phase iterator，以其upper-bound与mesh count一次`with_capacity`，随后按不变的`(sort_key, entity, source order, stable index)`完整键执行in-place `sort_unstable_by_key`。完整键已包含source-local stable index，等键项指向相同submission identity，不依赖stable sort。源码门禁先RED后GREEN，新增Rust source guard，`rustfmt --check`与scoped diff通过。

## P0瓶颈与路由

PERF-MVP-339：`mesh_recording`先用`has_transparent_sprite_submissions`扫描Transparent3d sprite items，实际mixed pass随后再次扫描并构建order；每frame/camera仍物化M+S items并执行O((M+S)log(M+S))比较。不能直接改线性merge：mesh command同sort-key按pipeline variant排序，sprite queue按entity排序，二者与mixed全序不一致。Render09联动Render02/14须让两个producer发布相同ordering contract与generation ranges，稳定帧复用，changed ranges做linear merge；Render17记录presence/build/sort/compare/allocation。

参考UE translucent pass的view draw-command owner与Bevy phase item/range消费边界；不改变mesh-before-sprite tie-break、entity稳定序或Transparent2d过滤语义。

## 验收

按mesh/sprites 0/1/1k/100k、比例0:1/1:1/10:1、equal sort/entity密度0/10/100%、cameras 1/8、stable/1% changed记录presence scans、item visits、Vec alloc/grow/capacity、sort calls/comparisons/temp bytes与CPU p50/p95/p99。当前Vec growth和stable-sort temp=0；最终stable mixed build/sort=0，changed工作近affected ranges且linear merge。current-source Cargo、ordering parity、F2 alpha blend/OIT像素、timestamp与DX12 RenderDoc通过前留在`pending.md`。
