---
title: Default HZB readback static performance review
date: 2026-07-17
status: static-reviewed-dynamic-pending
related_code:
  - zircon_runtime/src/graphics/pipeline/compile_options/default.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_execution.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/submit_compiled_scene_frame.rs
plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
---

# 默认 HZB readback 静态性能审查

## 可达性与阻塞链

- 默认 `RenderPipelineCompileOptions` 打开 `enable_hzb_occlusion_culling`，default forward+ 与 deferred pipeline 都包含 `BuiltinRenderFeature::Hzb`；因此它不是高级场景专属路径。
- 有 HZB candidate 时，`HzbOcclusionCuller` 把 GPU stats 复制到持久 readback buffer，但提交后立刻 `map_async` 并 `device.poll(wait_indefinitely)`。
- 同一帧还为 opaque、alpha-mask、advanced-PBR opaque、velocity 最多四个 indirect execution 分别创建 args MAP_READ buffer，并可能再创建 draw-count MAP_READ buffer。提交后 `collect_hzb_occlusion_indirect_args_readback_summary` 逐个调用 `collect`；每个 args/count 都独立启动 map 和 `poll(wait_indefinitely)`。
- cull params 每次执行还创建 upload buffer，execution bind group 也逐次创建；它们是次级 allocation 热点，应在去掉同步栅栏后用 capture/计数决定是否缓存。

最坏静态形态是一帧 1 次 stats blocking poll + 4 个 phase × args/count 最多 8 次 blocking poll，并伴随最多 8 个临时 readback buffer。即使首次等待已让 GPU 工作结束，后续 map/poll 仍增加 CPU 调度、driver round trip 与分配；逐 args CPU inspection 只是诊断数据，不应阻塞 GPU-driven 产品路径。

## 参考引擎对照

- Bevy `dev/bevy/crates/bevy_pbr/src/render/mesh_preprocess.wgsl` 和 GPU preprocess 路径在 GPU 上改写 indirect params 并直接供 draw 消费；不会为正常帧把每个 indirect args 全量读回 CPU。
- Fyrox `dev/Fyrox/fyrox-impl/src/renderer/occlusion/mod.rs` 的替代 query 路线采用异步回读和空间缓存；Zircon 继续用 HZB，但应迁移其“延迟、异步、可丢诊断样本”原则。

## 验收

1. HZB GPU stats 使用持久 2–3 槽 readback ring，map callback 在后续帧消费；not-ready 时保留上一份诊断并计数，不等待。
2. 正常产品帧不读回每个 indirect args/draw-count；只保留 GPU 侧 draw consumption。调试 inspector 需要时显式、低频、异步采样。
3. 1/100/10k candidate 的 WPR/Tracy/RenderDoc 对比包含 CPU wait、GPU bubble、copy、buffer allocation、cull stats age 与 draw parity。
4. params upload/bind-group allocation 在同步栅栏修复后按 profile 排序，避免未经数据的过度缓存。

## 路由

- `PERF-MVP-024` 已移交 Render 04，并要求复用 Render 16 的统一 async readback：`docs/plans/zircon_runtime/render/04/failure-2026-07-17-hzb-per-frame-blocking-readback.md`。

目录仍在 `pending.md`；当前源码动态 capture/测试被共享 Cargo reservation 延后。
