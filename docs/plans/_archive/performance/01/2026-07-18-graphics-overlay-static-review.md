---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/10-renderer-family.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/PrimitiveDrawingUtils.cpp
  - dev/bevy/crates/bevy_gizmos/src/retained.rs
  - dev/bevy/crates/bevy_gizmos_render/src/lib.rs
  - dev/bevy/crates/bevy_core_pipeline/src/core_2d/main_transparent_pass_2d_node.rs
tests:
  - overlay subtree 50 of 50 Rust files reviewed, 1703 current lines
  - disabled-sky volumetric allocation source guard RED then GREEN
  - WireOnly LoadStore opaque and transparent source guards RED then GREEN
  - rustfmt and scoped diff checks passed
  - current-source Cargo reservation 461d79d7bbe7445eb9645f3e8bfb7509 still not FIFO head
  - scale counters, F2/F4 pixels, timestamp and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics scene_renderer/overlay整个模块逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/overlay/**`当前50/50个Rust文件、1,703行，覆盖icon source/atlas、selection/wire/grid/gizmo/handle、preview sky、base/transparent scene、prepared buffers、pipeline构造、compiled/fallback调用入口与测试。该目录是基础编辑器viewport的直接产品路径，热点集中在每帧prepare、内部pass拆分、fallback mesh/sprite重建和optional GPU对象。

## 已直接止损

- `PreviewSkyPass`原先无论skybox是否Disabled，均先创建volumetric params buffer、binding entries与bind group，再仅跳过draw。现在以`SkyboxSettings::is_enabled()`门控该GPU对象组；Disabled仍执行必要的attachment clear/load/store pass，但buffer/bind-group create为0。
- opaque与transparent mixed base pass原先在`WireOnly`仍先构建forward bind group；后者还先构建mixed order和全部transparent sprite GPU buffers，开始render pass后才返回。现在仅当color/depth均为Load+Store时在所有昂贵工作前返回；Clear/Discard组合继续录制以保持render-graph attachment语义。
- 两组源码门禁均先RED后GREEN，纯helper行为测试已落盘；`rustfmt --check`和scoped diff通过。Cargo预约仍不是FIFO头，未把静态门禁冒充编译/动态验收。

## P0瓶颈与路由

- `record_overlays`对相同color/depth和render region依次开启selection、wireframe、grid、scene gizmo、handle最多5个Load+Store pass；其中4类line共享pipeline/bind group，gizmo只需在同pass切一次icon pipeline。PERF-MVP-333与Render01/10应融合为单个overlay pass并保留既有draw order。
- `prepare_buffers`每frame重建selection/wireframe/gizmo/handle CPU vertices及GPU buffers；每icon独立vertex buffer，atlas hit仍为每drawclone bind-group Arc。selection的O(S×M)和stable generation全量重建继续由PERF-MVP-333、Editor05/Render10/17统一retained owner解决。
- transparent mixed先生成每sprite GPU buffer，提交时又对每个sprite item在线性Vec中`find`，最坏O(S²)；同时与sprite stats、普通2D和OIT构成多套prepare owner。回链PERF-MVP-337/339，必须消费唯一prepared sprite artifact及dense/range handle。
- fallback `record_meshes`从`MeshDraw`再建9-phase command buffers，而compiled scene已有command artifact。回链PERF-MVP-383，fallback与compiled path只能共享同一phase arena，不保留第二command builder权威。
- `ViewportOverlayRenderer::new`无条件同步创建line/sky/icon pipelines、sky volumetric fallback与grid buffer；minimal/headless/无sky或无icons仍承担driver/GPU对象成本。回链PERF-MVP-356/390，由Render08/10按compiled feature generation按需single-flight。

参考UE primitive drawing的统一batched element提交与Bevy retained gizmo extraction/render owner；不把每个leaf的局部cache当最终架构。

## 验收

按display Shaded/WireOverlay/WireOnly、sky disabled/procedural/cubemap、selection/meshes/wire/gizmos/handles/icons/sprites各0/1/1k/100k、stable/1% changed记录CPU visits/find probes、Vec/Arc alloc、vertex bytes、command builds、GPU buffer/bind-group/pipeline create、upload、pass/draw及CPU/GPU p50/p95/p99。当前Disabled sky params buffer/bind=0、WireOnly LoadStore base/mixed pass及prepare=0；最终overlay pass≤1、stable retained rebuild/upload/GPU create=0、sprite lookup近O(S)、fallback command artifact额外build=0、minimal未请求pipeline create=0。Cargo、F2/F4 selection/wire/grid/gizmo/sky/transparent像素、timestamp与DX12 RenderDoc全部通过前留在`pending.md`，不进入`review.md`。
