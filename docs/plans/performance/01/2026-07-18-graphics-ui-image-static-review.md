---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/image.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_layout/21-gpu-submission-and-draw-pipeline.md
  - docs/plans/zircon_runtime/render/13-texture-pipeline.md
  - docs/plans/zircon_runtime/render/14-2d-stack.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
tests:
  - ui image root file 1 of 1 reviewed, 245 current lines
  - image/render/resource-streamer caller graph inspected
  - current-source Cargo reservation 461d79d7bbe7445eb9645f3e8bfb7509 still not FIFO head
  - image scale counters, editor pixels and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics UI image逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/ui/image.rs`当前1/1个Rust文件、245行，并追踪`render.rs`/rich-text inline image生产点、`ResourceStreamer::ui_texture`及主UI pass消费。该文件不做文件I/O或图片解码，texture已由streamer resident；热点是每draw CPU/GPU提交物化。

`prepare`每frame遍历全部image batches。每个可见image即使texture相同也独立clone `Arc<GpuTextureResource>`、创建2-entry bind group、生成6个完整NDC vertices、`create_buffer_init`一个GPU vertex buffer；render再逐image切scissor/binding/buffer并draw。稳定draw list没有generation命中，GPU object/create和vertex upload与image command数线性增长。

新增PERF-MVP-397：EditorLayout21/Render14用static indexed quad或vertex-index角点 + instance rect/UV/tint/scissor range；Render13按texture/view/sampler generation提供唯一resident binding handle；prepared UI generation持有可复用instance arena与ordered batch ranges，dirty range上传。clip/order边界仍决定batch，不允许跨非相邻命令重排，也不引入无generation裸WGPU cache。

此改动跨资源generation、order/clip与buffer lifetime，不属于安全局部补丁，本轮只写权威计划。

## 验收

按images 0/1/100/1k/10k、unique textures 1/10/100/1k、clip/scissor runs 1/10/100、stable/1% geometry/tint/texture change、hot reload/device loss记录texture Arc clone、bind-group/buffer create、CPU vertex/instance bytes、upload calls/bytes、scissor/bind switches、draw/batch与CPU/GPU p50/p95/p99。最终stable generation prepare/create/upload=0、bind group≤1/texture generation、CPU 6-vertex arrays=0、GPU payload为instance或≤4 vertices+6 static indices、changed近dirty commands，draw≤ordered compatible runs。Cargo、编辑器image/inline-image/clip/tint/hot-reload像素与DX12 RenderDoc通过前留在`pending.md`，不进入`review.md`。
