---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/scene_clear
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneRendering.cpp
  - dev/bevy/crates/bevy_core_pipeline/src/core_3d/main_opaque_pass_3d_node.rs
tests:
  - scene_clear subtree 4 of 4 Rust files reviewed, 363 current lines
  - existing offscreen color-depth combined clear test reviewed
  - current-source Cargo, pass counters, F2 camera-stack/split pixels, timestamp and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics scene_renderer/scene_clear整个模块逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/scene_clear/**`当前4/4个Rust文件、363行，覆盖clear plan消费、uniform、shader、三条pipeline、region pass录制和offscreen test。本地实现已持久化GPU资源，在无clear/空region早退，并把color+depth合并成一个pass；主要瓶颈在camera attachment policy没有区分full-target与partial region。

## P0瓶颈

PERF-MVP-394：为避免WGPU attachment load clear误清整个split-view texture，`ViewportCameraStackAttachmentPolicy`把scene color/depth首次Clear一律改Load，`execute_compiled_scene_graph_stages`随后在每camera graph前调用`record_frame_clear`。这对partial/split/overlay正确，但full-target首camera也额外开启region render pass、画fullscreen triangle，color clear还写16B uniform。构造期始终创建color/color+depth/depth三条pipeline，哪怕产品只需可融合full-target clear。

Render09须把target coverage与stack-first identity纳入clear intent，Render01在full-target路径把color/depth恢复为各自首次attachment write的LoadOp::Clear；只有partial region保留draw clear，且color+depth继续≤1 pass。region-clear pipelines按实际partial-clear feature single-flight；Render17记录pass/draw/write/pipeline与GPU timestamp。

参考UE scene rendering和Bevy opaque pass attachment load-op边界；不把partial viewport改成全view clear，不改变overlay `clear_depth`、MSAA transparent clear或preview clear color语义。

## 验收

按cameras 1/8、regions full/2×2 split/overlap overlay、clear skybox/color/depth/none、MSAA1/4记录region-clear pass/draw、uniform writes、first-write load ops、pipeline creates、attachment bytes和CPU/GPU p50/p95/p99。full-target首camera要求region pass/draw=0且color/depth首次写正确Clear；partial color+depth≤1 pass，无clear=0，未请求partial clear时三pipeline=0。current-source Cargo、F2 camera stack/split/overlay逐像素、timestamp与DX12 RenderDoc通过前留在`pending.md`。
