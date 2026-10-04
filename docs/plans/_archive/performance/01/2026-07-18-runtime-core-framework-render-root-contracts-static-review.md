---
related_code:
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot_streams.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_execution_draw.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/extract_param.rs
  - dev/bevy/crates/bevy_render/src/extract_component.rs
  - dev/bevy/crates/bevy_render/src/extract_resource.rs
tests:
  - render root remaining six of six Rust files reviewed
  - focused scene and editor viewport packet callers traced
  - transform revision source guard RED at two calls and GREEN at one call per mesh entity
  - rustfmt and scoped git diff check passed
  - current-source Cargo, scale counters, F2/F4 traces and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime render root契约逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读render root最后`backend_types.rs`、`mod.rs`、`scene_extract.rs`、`virtual_geometry_debug_snapshot.rs`、`virtual_geometry_debug_snapshot_streams.rs`与`virtual_geometry_execution_draw.rs`当前6/6个Rust文件、2,259行，并聚焦追踪scene与editor viewport packet生产调用链。`mod.rs`仅wiring；execution draw为定长DTO；VG debug stream encode/decode当前无产品caller，只在显式诊断/测试使用，接入产品UI时必须按capture generation触发，不能按帧做pack→decode。巨型owned `RenderStats`及graph report clone继续归PERF-MVP-324/343，不另建重复权威。

## PERF-MVP-349：editor兼容render packet先深clone整个World

`World::build_viewport_render_packet(&self, ...)`为了运行`RenderExtract`内部系统，先执行`let mut world = self.clone()`，再遍历mesh、light和camera构包。editor `SceneViewportController::build_render_snapshot`经`build_render_packet`在viewport snapshot路径直接调用，因此场景节点、组件、资源索引和派生状态在真正extract前已整World复制；随后每个mesh primitive还重复计算同一transform revision并clone morph weights/layer mask。

本轮先把无风险的transform hash移到primitive循环外：同一entity无论primitive数量只hash一次，源码守卫经历2→1 RED/GREEN。World ownership、RenderExtract system mutation与editor snapshot generation属于架构边界，Runtime07/Editor05应把派生更新放进明确schedule并让extract只读live world，或发布generation-owned render-world artifact；stable generation复用artifact，禁止保留clone-World兼容双权威。Bevy `Extract<P>`在`ExtractSchedule`中以read-only system parameter访问`MainWorld`，visible component批量抽取还复用上一帧capacity；resource只有changed时更新，证明无需为允许extract而复制整个simulation world。

## 验收要求

按nodes/components/meshes/primitives/lights 0/1/1k/100k，editor idle/camera move/selection/1% scene change记录World clone bytes、extract builds、component visits、transform hash count、morph/layer clone bytes与CPU/RSS p95：World clone bytes=0；transform hash≤mesh entity count而非primitive count；stable generation full extract build=0；changed generation的`RenderExtract` stage恰好一次。scene system mutation、camera/layer/LOD、overlays、resource handles、packet ordering与F2/F4产品像素必须等价。current-source Cargo、规模counter、editor产品trace和RenderDoc未完成前，本批留在`pending.md`。
