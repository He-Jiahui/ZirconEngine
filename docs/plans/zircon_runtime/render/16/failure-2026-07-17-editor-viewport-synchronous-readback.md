---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: editor-viewport-synchronous-readback
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/render/16-compute-neural.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/render/16
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/viewport/submit_extract.rs
  - zircon_editor/src/ui/retained_host/viewport/poll_image.rs
  - zircon_editor/src/ui/retained_host/viewport/viewport_state_ensure_viewport.rs
  - zircon_editor/src/ui/retained_host/viewport/world_space_ui.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_target/finish_viewport_frame.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba.rs
tests:
  - editor viewport steady-state no synchronous GPU poll guard
  - async readback ring backpressure and generation test
  - GPU texture interop resize and device-loss product test
---

# Render16：Editor viewport 常规帧同步整帧 readback

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F4 retained viewport 到 RenderFramework/SceneRenderer/WGPU readback 的产品调用链静态审查
- 修复责任计划：`docs/plans/zircon_runtime/render/16-compute-neural.md`
- 共同责任：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 交接原因：Render16 已声明统一异步 readback owner；Editor01 需冻结 viewport GPU texture import/composition 契约，不能只在 UI 端少 clone 一次。

## 失败现象与复现证据

Editor retained viewport 用 `submit_frame_extract_with_ui` 提交常规帧。render submit 进入 `render_frame_with_pipeline` 后，无条件通过 `finish_viewport_frame -> read_texture_rgba` 把 final texture 读回 CPU。该 helper 每帧创建 staging buffer、encoder 和 MPSC，提交 copy，调用 `device.poll(wait_indefinitely)`，再复制到新的 RGBA `Vec`。随后 `record_capture` 每帧生成 graph dump 并深 clone compiled pipeline，`capture_frame` clone 整个 RGBA，Editor `SharedPixelBuffer::clone_from_slice` 又复制一次。

1080p RGBA payload 单帧约 7.9 MiB，60 FPS 约 475 MiB/s，且同步 poll 强制 CPU/GPU 栅栏。submit 期间还持有 framework operation/state guard，GPU stall 会放大多 viewport 与其他 render 操作的排队。

26-file retained viewport 审查进一步确认，Editor controller 的共享 mutex 跨 `capture_frame`、`SharedPixelBuffer::clone_from_slice`、render-framework resolve、viewport destroy/create 与 submit 持有；size 每次变化立即 destroy/create，resize burst没有 latest-value coalescing。旧 `poll_image` 在任何 capture/import error 后还会把 cached image作为“新图”返回，引发 stale redraw；本轮已直接修正为错误返回 `None` 并保留缓存/诊断，同时把 GI profile env解析缓存一次，但这不减少正常帧同步 readback。

## 最低共享层根因

`submit_frame_extract` 把“渲染产品帧”和“同步截图”合成一个返回 `ViewportFrame` 的语义；Editor 没有可直接交给 retained UI compositor 的 GPU texture generation/handle contract，也没有有界 async readback fallback。

## 架构修复验收

- 同 GPU device/backend 时，viewport product 以稳定 texture handle/view + generation/lifetime fence 交给 UI；常规帧没有 MAP_READ、`wait_indefinitely` 或 CPU RGBA clone。
- 无法 GPU 互操作时使用有界 2–3 槽 async staging ring；UI 取最新 ready frame，积压有 drop/coalesce 计数且不阻塞 render/main thread。
- screenshot/headless/pixel-test 保留显式 capture/readback API；submit 与 capture 分离，现有产品测试可迁移而不丢像素验收能力。
- graph dump 与 compiled pipeline snapshot 按 pipeline generation 缓存/共享，正常帧不做文本重建和 compiled graph 深 clone；诊断请求保留相同内容。
- 1080p 30/60/120 FPS、resize、device loss、多 viewport 做 WPR/Tracy/RenderDoc；报告 GPU wait、copy bytes、allocation、main-thread p95 和 frame age。
- controller 锁不跨 framework/GPU调用或整帧 import；resize burst按 frame latest-value 合并，destroy/create≤1/frame；capture/import error不重发 stale generation。

## 禁止临时方案

- 不得只把同步 readback 移到另一个主线程函数或每 N 帧阻塞一次。
- 不得建立无界 pending map/readback 队列，或跨 ABI 暴露未定义生命周期的裸 WGPU pointer。
- 不得删除 screenshot/pixel-test 能力来伪造产品帧加速。

## 修复结果与回传

Open state: `待 Render16 + Editor01 分离 product texture 与 explicit capture，并完成动态验收`。
