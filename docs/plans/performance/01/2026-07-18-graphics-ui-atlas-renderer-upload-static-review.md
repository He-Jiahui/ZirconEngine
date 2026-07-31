---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_texture_upload
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
  - docs/plans/zircon_runtime/render/13-texture-pipeline.md
  - docs/plans/zircon_runtime/render/14-2d-stack.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
tests:
  - atlas_renderer 6 of 6 Rust files reviewed, 1456 current lines
  - atlas_texture_upload 6 of 6 Rust files reviewed, 1154 current lines
  - scene_renderer ui total 71 of 71 Rust files reviewed, 16101 current lines
  - rustfmt and git diff checks remain green for prior changed UI slices
  - focused Cargo reservation c5e7a6ccdba740b59c223c2a8307de63 remains queued, not FIFO head
  - F2 text pixels and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics UI atlas renderer/upload逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`atlas_renderer` 6/6文件、1456行及`atlas_texture_upload` 6/6文件、1154行。至此`scene_renderer/ui/**`当前71/71个Rust文件、16101行已逐文件完成静态性能审查。

GPU texture owner已有正确的基础生命周期：atlas texture/view/bind group仅在size、layer count或storage format变化时替换；upload binding按staging index直接定位，并校验page key/generation、row stride、byte length与range；missing page、generation mismatch和face invalidation均fail-closed requeue，稳定无request时不写texture。

主要剩余热点在draw prepare。每个storage pass每帧都对`GlyphAtlasGpuDrawPlan.vertices`执行`create_buffer_init`，并clone完整draw command Vec；mixed alpha/RGBA storage又逐pass构建prepared upload、frame request/binding plan与pass report Vec。render对每draw在线性pipeline resource Vec中find，report aggregation对少量pass多次扫描。constructor还在无native text时创建1x1 atlas、shader/layout/sampler，optional pipeline虽按contract懒建。核心问题归PERF-MVP-231和PERF-MVP-390，不用局部无generation cache掩盖。

Text04计划文件当前被另一会话租约占用，本轮不并发写；主性能计划、Render13/14/17已记录交接。最终由Text04发布single mixed-storage generation artifact，Render14维护persistent vertex/instance arena与dense pipeline handle，Render13只消费prepared dirty page commands。

## 验收

按glyphs 0/1/100/10k、storage alpha/color/mixed、pages 1/16/256、stable 300 frames/1% glyph change/face reload/device loss记录prepared upload/frame/binding/report builds、source bytes clone、vertex buffer create/bytes、draw command clone、pipeline probes、texture/bind-group create、write calls/bytes与CPU/GPU p50/p95/p99。最终stable generation上述prepare/create/clone/write全部为0；changed近dirty slots/ranges，pipeline lookup O(1)，missing/generation/face失效计数与像素一致。Focused Cargo、F2 native/SDF parity、Softbuffer/WGPU和DX12 RenderDoc通过前留在`pending.md`，不进入`review.md`。
