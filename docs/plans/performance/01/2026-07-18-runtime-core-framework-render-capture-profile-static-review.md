---
related_code:
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/core/framework/render/framework_error.rs
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/core/framework/render/module_identity.rs
  - zircon_runtime/src/core/framework/render/overlay.rs
  - zircon_runtime/src/core/framework/render/profile.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capture_frame
  - zircon_runtime/src/graphics/runtime/render_framework/graphics_debugger_capture
  - zircon_editor/src/ui/retained_host/viewport/poll_captured_frame.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - docs/assets-and-rendering/render-framework-architecture.md
tests:
  - capture framework overlay profile root six of six Rust files reviewed
  - graphics debugger capture directory six of six Rust files read as focused caller validation
  - source-guard RED to GREEN for generation filtering before RGBA clone
  - public render-framework architecture documentation updated
  - rustfmt and scoped git diff check passed
  - renderdoccmd explicit invocation failed because executable is not installed or on PATH
  - current-source Cargo, scale counters, F2 traces and RenderDoc capture pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime render capture/framework/overlay/profile逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`capture.rs`、`framework_error.rs`、`framework.rs`、`module_identity.rs`、`overlay.rs`与`profile.rs`当前6/6个Rust文件、1,000行；另完整读取graphics debugger capture目录6/6并追踪wgpu capture、editor import和dynamic bridge调用。capture scalar report、module identity和error-only String不形成稳态热点；profile bundle只在启动/config使用且集合≤16，无独立frame热点；overlay owned Vec风险已由PERF-MVP-333负责，不重复编号。

## PERF-MVP-023：stale editor poll仍深拷贝整帧RGBA

`WgpuRenderFramework::capture_frame`原在framework state mutex内无条件clone viewport最后一帧的完整RGBA；retained editor每host tick调用后才比较generation，未提交新帧也会复制1080p约8 MiB，再在`SharedPixelBuffer::clone_from_slice`复制一次。dynamic runtime bridge同样在拿到owned frame后才判stale。

本轮TDD新增向后兼容的`RenderFramework::capture_frame_if_newer`合同：默认实现保持现有backend兼容，wgpu override在clone前比较stored generation；editor poll和dynamic bridge传入最后消费generation。stale tick现在在锁内只读scalar并返回None，RGBA clone bytes=0。公开合同同步写入`docs/assets-and-rendering/render-framework-architecture.md`。

剩余新帧仍在全局state lock内clone完整RGBA，editor随后再次复制；上游还同步GPU readback、每帧graph dump与compiled pipeline clone。Editor01/EditorUI08/Render17继续按PERF-MVP-023交付短锁capture handle、GPU texture import或有界async readback ring、generation-owned graph/pipeline artifact；本轮不以stale快路冒充最终关闭。

## PERF-MVP-324：query_stats公开合同返回巨型owned snapshot

`RenderFramework::query_stats`返回`RenderStats` owned value，wgpu实现会复制大量String/Vec/report；这与已登记的541-series diagnostics全量采集和generation/delta缺失同根。Render17/Runtime07继续负责按需domain snapshot与共享generation，不新建ID。

## RenderDoc可达性与当前工具证据

request→pending viewport→submit前`start_graphics_debugger_capture`→operation lock保护下释放state mutex并stop/poll的产品链真实存在，status也明确“hook available不等于RenderDoc attached”。本机显式执行`renderdoccmd.exe --help`返回PowerShell command-not-found，PATH、`C:\Program Files\RenderDoc`常见路径与workspace均未找到可执行文件；因此本轮只能记录tool-unavailable，不能声称GPU capture或瓶颈定位完成。工具缺失不阻塞静态审查与代码修复。

## 验收要求

按720p/1080p/4K、new frame 0/1、polls per frame 1/10/100记录framework lock hold、RGBA clone bytes、CPU copy bytes、GPU readback stall与image import p95：stale poll clone/copy=0，最终new frame CPU完整copy≤1且正常editor viewport走GPU texture或bounded async ring。`query_stats`按UI poll 1/10/60 Hz与stats payload规模记录clone bytes/lock p95，同generation全量clone=0。安装可用RenderDoc后用真实MVP editor scene抓F2 frame，核对pass/draw/upload/buffer和GPU p95；current-source Cargo、editor polling/dynamic bridge、capture failure/unknown viewport回归全部通过前，本批留在`pending.md`。
