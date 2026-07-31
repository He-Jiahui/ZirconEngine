---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_runtime/render/10-renderer-family.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/PrimitiveDrawingUtils.cpp
  - dev/bevy/crates/bevy_gizmos/src/retained.rs
  - dev/bevy/crates/bevy_gizmos_render/src/lib.rs
tests:
  - primitives subtree 51 of 51 Rust files reviewed, 1279 current lines
  - Shaded wireframe and WireOverlay selection-index source guard RED then GREEN
  - rustfmt and scoped diff checks passed
  - current-source Cargo reservation 461d79d7bbe7445eb9645f3e8bfb7509 pending behind FIFO
  - scale counters, F2/F4 overlay pixels, timestamp and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics scene_renderer/primitives整个模块逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/primitives/**`当前51/51个Rust文件、1,279行，覆盖SceneUniform/SH9 ABI、line/icon vertices、grid、selection、wireframe、gizmo、handles、fallback math和GPU buffer helpers。大多数文件是小型append/ABI叶；实际热点由每帧overlay prepare调用的CPU geometry builders与`create_buffer_init` helpers形成。

## 已直接止损

`ViewportOverlayRenderer::prepare_buffers`每帧无条件调用wireframe builder，而`WireframePass::record`在Shaded才早退。原builder因此在默认Shaded仍遍历全部mesh/model/wire segments、变换所有端点、增长Vec并创建GPU vertex buffer；selection HashSet也无条件构造，WireOverlay并不读取它。

现在builder在Shaded于selection构建和mesh循环前返回空Vec，`build_line_buffer`随之返回None；selection HashSet只在WireOnly构建。源码门禁先RED后GREEN，新增Rust source guard，`rustfmt --check`与scoped diff通过。

## P0瓶颈与路由

- PERF-MVP-333：selection对每个highlight执行`frame.meshes().iter().find`，最坏O(selections×meshes)，命中后每frame重做model bounds变换；wire modes仍展开全部wire segments；gizmo/handle每frame执行frustum/ring/arrow trig与顶点构建，per icon单独创建vertex buffer，line/icon buffers均无持久capacity。Render10/Editor05须消费scene/overlay generation的dense index和retained geometry，camera-facing icon/anchor用instance transform，GPU arena只dirty upload。
- PERF-MVP-346：`SceneUniform::from_frame`构建jittered/unjittered matrix pair与inverse，`previous_motion_view_projection`又调用`VelocityCameraParams::from_cameras`重建current/previous pair和inverse。应消费唯一prepared-camera artifact。
- grid的84 vertices只在renderer构造期创建一次，不是frame hotspot；fallback converters与vertex layouts为O(1) Copy/ABI合同，不新增任务。

参考UE PrimitiveDrawingUtils和Bevy retained gizmo/render extraction的geometry owner边界；保留selection/wire colors、48-segment rings、camera-facing fallback icons、finite fallback和GPU vertex ABI。

## 验收

按display Shaded/WireOverlay/WireOnly、meshes/selections/wire segments/gizmos/handles/icons 0/1/1k/100k、cameras 1/8、stable/1% changed记录mesh/model probes、matrix/trig、Vec/HashSet alloc/grow、vertex count/build bytes、GPU buffer creates/uploads/draws与CPU/GPU p50/p95/p99。当前Shaded wire visits/vertices/buffer=0、WireOverlay selection HashSet=0；最终selection近O(selected)、stable overlay rebuild/upload/buffer create=0、changed近delta，camera matrix pair≤1/generation。current-source Cargo、F2/F4 selection/wire/gizmo/handle像素、timestamp与DX12 RenderDoc通过前留在`pending.md`。
