---
related_code:
  - zircon_editor/src/ui/retained_host/viewport
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/16/failure-2026-07-17-editor-viewport-synchronous-readback.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore
  - dev/godot/servers/rendering
tests:
  - capture-error stale-image regression RED then implementation
  - GI environment process-cache source boundary RED then GREEN
  - existing viewport create/resize/capture/world-UI/job suites and Windows focused Cargo pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor Retained Viewport 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`zircon_editor/src/ui/retained_host/viewport` 当前共 **26** 个 Rust 文件，包含 production controller/state/readback/world-UI 与模块内测试，已逐文件阅读 **26/26**。动态 Cargo、current-source WPR/RenderDoc cold+warm capture 与 resize/error storm 尚未完成，因此继续留在 `pending.md`。

## 主要结论与直接修复

- `poll_image()` 每 host tick调用 `capture_frame()`；当前 runtime capture 最终进入同步 staging/readback白名单。controller 在 `Mutex<ViewportState>` 内完成 framework call 与整帧 `SharedPixelBuffer::clone_from_slice`，主线程/GPU stall会阻塞 submit、world-UI pointer和其他 controller消费者。
- 旧错误路径返回 `latest_image.clone()`，把已发布图当新图反复触发 redraw/upload。已加 capture-error行为回归并改为 None；缓存仍保留，诊断仍由 `take_error()` 取得。成功新帧存一份 image clone并直接返回原 image，少一次 clone。
- `apply_editor_viewport_render_defaults()` 旧实现每 frame `std::env::var` + trim/lowercase。已加源码边界并用 `OnceLock<Option<RenderHybridGiProfile>>` 进程级解析一次；启动前环境配置语义不变。
- size变化立即 destroy旧 viewport、清 generation/image并 create新 viewport/quality profile；连续 resize没有帧级 coalesce。与同步 readback共同归 PERF-MVP-023/Render16+Editor01。
- `submit_extract_with_ui()` 在 controller lock内 merge world-space submissions、构造 render commands并调用 framework submit。world-space unchanged frame仍分配 style/font/color strings；pointer capture clone完整 submission。归 PERF-MVP-121/EditorUI08+Render16。
- 默认 Hybrid GI enabled 与 custom budgets属于产品质量选择；性能验收必须按 fully-dynamic/indoor/open-world/custom分别记录，不能通过静默关闭功能“优化”。

## 待验收

focused suite覆盖 lazy framework job、create/resize、aspect、new/same/none/error capture、world-UI render/capture/cancel。WPR/RenderDoc覆盖1080p cold/warm、30/60/120 FPS、resize storm与多 viewport，记录同步 wait/readback bytes、controller lock hold/wait、frame age、stale publish、viewport recreate与world-UI alloc。通过前不进入 `review.md`。
