---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
reference_sources:
  - dev/slint/internal/backends/winit/winitwindowadapter.rs
  - dev/slint/internal/backends/winit/event_loop.rs
  - dev/slint/internal/core/partial_renderer.rs
tests:
  - host window lifecycle, diagnostics, scale-factor and redraw state tests
  - native pointer/keyboard/popup/text-input tests
  - current-source Windows Cargo pending
  - 1k-event redraw/hover/window-query/copy trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor host window逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`window.rs` + `window/**`共 **38** 个Rust文件、**1,771** 行，已逐文件阅读 **38/38**。覆盖winit lifecycle/events/platform input、redraw/present、presentation snapshot/hover注入、window handle、text edit/keyboard及测试。当前源Cargo和产品event storm尚未完成，因此仍留在`pending.md`。

## 已有正确边界

Event loop使用Wait语义，不在`about_to_wait`无条件tick或present；redraw区分需要frame update与paint-only，pending request在RedrawRequested时一次take。IME allowed只在active状态变化时调用native window，close callback先释放state borrow再重入。Presenter失败有GPU→Softbuffer fallback，空/无效文本和无效damage会提前退出。

## 热点与计划

- PERF-MVP-164：`host_presentation_from_state`先深clone完整presentation；hover id非空时，`apply_template_hover_to_nodes`对每个node collection无条件收集完整Vec，即使没有control匹配也clone/drop所有rows。调用还遍历workbench、四dock和全部floating windows，匹配option/menu后再clone整张row表。Hover应是独立transient generation，通过stable control index和old/new frame形成damage，不得写回结构ModelRc。
- PERF-MVP-165：pointer/key/IME产生redraw时原先无论pending状态都调用winit `request_redraw`；高频事件在单次RedrawRequested前重复跨native边界。本轮已按Slint `pending_redraw` edge让queue用move/replace合并并返回false→true transition，pointer/external/resize只在首次transition schedule，新增queue边沿测试；当前源Cargo待验收。
- PERF-MVP-166：`about_to_wait`每次调用`surface_size/scale_factor/is_maximized/outer_position`并无条件覆写state。Window状态应由resize/move/DPI/lifecycle事件更新，startup只sync一次；缺失事件只能用显式低频reconcile并计数。
- PERF-MVP-167：insert/backspace原先把current SharedString复制为String，修改后又完整clone给state，再把原String转SharedString发callback；focus亦重复clone。本轮已让编辑结果只转换一次SharedString并共享给state/callback，focus只move一次，control过滤直接extend而不建临时String；最终仍由EditorUI03接runtime owned edit buffer/range delta，避免长度N逐键输入的O(N²)总复制。Rustfmt已过，当前源Cargo待验收。

## 关联项

Present仍通过`get_host_presentation`深clone全树并在成功后改写diagnostics overlay，分别由PERF-MVP-147/149负责；单矩形damage由PERF-MVP-163负责；`export_present_artifacts`每present调用的实际开销留给`profiling_artifacts`模块审查。`SystemTime::now`输入时间戳与platform translation需结合PERF-MVP-052的高频输入产品trace再决定是否独立整改，不以静态猜测改时间语义。

## 动态验收

对1/1k pointer move、typing、IME与外部redraw记录structural clone bytes、hover node/row visits、native request_redraw count、about-to-wait native property queries/state writes、text copied bytes与main-thread p50/p95。同一drain周期native schedule=1；hover结构row visit/Vec build=0；idle/input-only window poll=0；10k追加文本总复制近线性。Resize/move/DPI/maximize、IME、popup、hover、frame-update scenario和GPU/Softbuffer pixels必须等价。
