---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass
  - zircon_runtime/src/graphics/scene/scene_renderer/anti_alias
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/06-temporal-pipeline.md
  - docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
tests:
  - prepass subtree 3 of 3 Rust files reviewed, 5 current lines
  - scene-renderer anti_alias subtree 3 of 3 Rust files reviewed, 10 current lines
  - current-source Cargo, F2 prepass and AA pixels, timestamp and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics scene_renderer prepass与anti_alias wiring逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/prepass/**`当前3/3个Rust文件、5行，以及`scene_renderer/anti_alias/**`当前3/3个Rust文件、10行。两者都是编译期wiring：prepass只分层导出`NORMAL_FORMAT=Rgba8Unorm`，anti_alias只导出FXAA/SMAA pass name、executor ID和WGSL entry point常量。没有循环、容器、I/O、锁、线程、GPU对象或每帧函数，因此不新增性能任务。

真实prepass成本在mesh normal/depth pass、pipeline/binding与command replay，继续由PERF-MVP-368/383和Render02负责；真实AA成本在post graph、TAA reactive mask、FXAA/SMAA executors和pipeline构造，继续由PERF-MVP-350/370/371与Render06/07/17负责。常量模块必须保留ABI/名称一致性，不为减少文件数做无收益迁移。

## 验收

current-source Cargo、F2 normal/depth/velocity与AA none/FXAA/SMAA/TAA像素对拍、pass/timestamp counters和DX12 RenderDoc完成前，这两个目录仍随其产品owner留在`pending.md`；不以15行常量静态阅读替代动态产品验收。
