---
title: Editor viewport render static performance review
date: 2026-07-17
status: static-reviewed-dynamic-pending
related_code:
  - zircon_editor/src/ui/retained_host/viewport
  - zircon_runtime/src/graphics/runtime/render_framework
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_target/finish_viewport_frame.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba.rs
plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
---

# Editor viewport render 静态性能审查

## 已证实的产品调用链

1. `RetainedViewportController::submit_extract` 调用 `RenderFramework::submit_frame_extract_with_ui`。
2. `submit_runtime_frame` 在持有 framework operation guard 和 mutable renderer state 时调用 `SceneRenderer::render_frame_with_pipeline`。
3. 该入口完成 GPU scene submit 后无条件调用 `finish_viewport_frame`；后者调用 `read_texture_rgba`。
4. `read_texture_rgba` 每帧新建 MAP_READ staging buffer、command encoder 与 `std::sync::mpsc`，单独提交 texture-to-buffer copy，随后 `device.poll(wgpu::PollType::wait_indefinitely())` 阻塞等待 GPU，并逐行复制到新 `Vec<u8>`。
5. `record_capture` 同一帧还会执行 `compiled_pipeline.graph.dump().to_text()` 并深 clone `CompiledRenderPipeline`（含 Vec 与 compiled graph）；`capture_frame` 再 clone `CapturedFrame` 的整帧 RGBA。
6. Editor `poll_image` 用 `SharedPixelBuffer::clone_from_slice` 第三次复制 RGBA，再 clone 保存 retained `Image`；CPU image 随后进入 UI 绘制/上传路径。

所以 editor 主 viewport 的常规帧不是 GPU resident composition，而是同步 GPU→CPU readback。1080p 单帧约 7.9 MiB RGBA，60 FPS 每一次整帧复制约 475 MiB/s；mapped staging→Vec、capture clone 与 UI pixel-buffer clone 三次理论 CPU 像素复制量合计约 1.4 GiB/s，尚未计入 staging 对齐、retained image 与 UI re-upload。更关键的是 `wait_indefinitely` 破坏 CPU/GPU overlap，并延长 framework 全局操作/状态锁持有窗口。

## 参考引擎对照

- Godot `dev/godot/scene/main/viewport.cpp` 的 `Viewport::get_texture()` 返回 `ViewportTexture` GPU resource；`ViewportTexture::get_rid()` 暴露 texture proxy。CPU `get_image()` 是独立的显式操作，不是 viewport 每帧显示的默认路径。
- Zircon 当前已经有 `present_frame_with_pipeline(..., ViewportSurface)` 的 GPU present 路径，证明 renderer 并非只能读回；Editor 需要的是可导入 UI compositor 的 texture handle/registry contract，或在无法共享 device/texture 时使用有界 async readback fallback。

## 修复验收建议

- GPU 同 device：Editor viewport UI 直接消费稳定 texture handle/view，按 generation 做 lifetime fencing；不得复制 WGPU handle 穿越未冻结 ABI。
- 跨 device/backend fallback：2–3 槽 staging ring，map callback 异步完成，UI 只取最新 ready generation；积压时覆盖/丢弃旧帧而不是阻塞主线程。
- screenshot、pixel test、headless capture 保留显式同步 capture API，不能把产品 viewport 与测试 readback 混成同一 submit 语义。
- compiled pipeline 与 graph dump 按 pipeline generation 缓存；正常 submit 不重建文本 dump，诊断读取才物化/共享，不再每帧深 clone graph。
- WPR/Tracy 记录 submit、GPU wait、readback map、UI import/upload；RenderDoc 对比修复前后 copy/pass 与稳定帧。

## 路由

- `PERF-MVP-023` 已移交 Render 16 的统一异步 readback owner，并要求 Editor 01 共同冻结 GPU texture interop：`docs/plans/zircon_runtime/render/16/failure-2026-07-17-editor-viewport-synchronous-readback.md`。

本记录只完成上述调用链静态验明；当前源码产品 WPR/RenderDoc 与修复后数据尚未获得，目录继续保留在 `pending.md`。
