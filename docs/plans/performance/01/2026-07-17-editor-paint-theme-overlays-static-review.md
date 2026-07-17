---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_close_prompt.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_close_prompt/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_debug_reflector_overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_diagnostics.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/godot/scene/gui/button.h
  - dev/godot/scene/gui/button.cpp
  - dev/slint/internal/core/partial_renderer.rs
tests:
  - paint theme token projection tests
  - debug reflector and diagnostics overlay geometry/pixel tests
  - current-source Windows Cargo pending
  - 1/1k/10k node theme-lock and generation trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor paint theme/workbench/overlay逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`paint_theme`共 **6** 个Rust文件、**473** 行；`paint_workbench`共 **3** 个、**105** 行；`paint_close_prompt`共 **5** 个、**98** 行；`paint_debug_reflector_overlay`含相邻测试共 **4** 个、**156** 行；`paint_diagnostics`含相邻测试共 **6** 个、**149** 行。合计 **24/24** 个文件、**981** 行已逐文件阅读。当前源Cargo、主题锁计数和产品trace未完成，全部仍留在`pending.md`。

## 已有正确边界

Workbench入口只选择componentized或fallback renderer；close prompt、debug reflector和diagnostics overlay都先验证visibility/clip，overlay primitive按kind直接投影颜色，未发现独立的无界队列或高阶算法。Damage repaint会保留backbuffer并以active clip限制primitive写入。Overlay文本测量与完整presentation damage fixed-point的重复成本已分别由PERF-MVP-156与PERF-MVP-149覆盖。

## 热点与计划

PERF-MVP-161：`current_host_palette()`和`current_host_metrics()`每次都读取全局`RwLock`。静态调用面已有 **199** 处palette访问（86文件）和 **83** 处metrics访问（41文件），material primitive可在一个node内多次读取相同palette。`apply_host_appearance_from_tokens`还按palette、metrics、typography顺序更新三把锁，所以并发render既支付per-style同步成本，也可能观察到混合theme generation。

该问题不适合在282个调用点增加consumer-local cache。EditorUI08应一次生成`HostThemeSnapshot { generation, palette, metrics, typography }`并原子发布；frame/command build获取一次稳定handle，再经paint context把借用值传给style helper。主题变更作为presentation/style dirty generation精确失效；正常node paint不再触碰全局锁。

## 参考引擎约束

Godot `Button::ThemeCache`在theme更新时集中填充style/font/color/icon，draw和measure读取本地cache；Slint partial renderer用`PropertyTracker`只重新求值dirty item rendering。Zircon可以使用generation-owned共享snapshot而非逐Control cache，但必须保持相同原则：更新时投影、绘制时只读稳定值，不能在每个primitive里重新同步查询全局theme。

## 动态验收

对1/1k/10k普通、MUI、popup与diagnostics nodes记录每frame theme snapshot acquisition、palette/metrics lookup、RwLock wait、style projection count和CPU p50/p95。稳定frame全局theme lock≤1且per-node=0，theme change原子发布一次并只重绘依赖style的generation；主题切换前后layout、colors、text smoothing与GPU/Softbuffer/screenshot pixels必须等价，不出现mixed generation。
