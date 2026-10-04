---
related_code:
  - zircon_editor/src/ui/retained_host/document_tab_pointer
  - zircon_editor/src/tests/host/retained_document_tab_pointer
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-viewport-toolbar-surface-rebuild-storm.md
reference_sources:
  - dev/slint/internal/backends/qt/qt_widgets/tabwidget.rs
  - dev/godot/scene/gui/tab_bar.cpp
tests:
  - measured-frame clone/no-op source boundary RED then GREEN
  - existing retained document-tab pointer route suites and Windows focused Cargo pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor Retained Document Tab Pointer 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`zircon_editor/src/ui/retained_host/document_tab_pointer` 当前共 **19** 个 Rust 文件，已逐文件阅读 **19/19**，覆盖 model projection、root/floating surfaces、measured frames、tab/close route、surface 构建与 callback。动态 Cargo、1k-click storm 与 route parity 尚未完成，因此继续留在 `pending.md`。

## 主要结论与直接修复

- layout builder 会为 root 与每个 floating window 克隆 tab instance ids；这应只在 document/floating-window generation 变化时构建，仍由 PERF-MVP-106 的 generation-driven slow recompute 约束。
- activate/close callback 每次先调用 `update_measured_frame()`；旧实现即使 frame 与上次完全相同也覆写并 `rebuild_surface()`，把一次稳态点击放大为全部 surface/tab/close node、path、dispatcher 与 route map 重建。
- 已先加入源码边界回归并确认旧源码为 RED，再增加相同 measured frame no-op。首次测量、host resize 或布局变化仍重建，路由语义不变。
- `rebuild_surface()` 旧实现 `.cloned()` 整个 measured-frame vector；现直接借用 slice 并逐项 `copied().flatten()`，缺失 store/entry 时仍按最小宽度 fallback。
- 首次测量仍是 O(all tabs) full rebuild；若动态 click/resize trace 显示为热点，EditorUI08 应把单 tab frame/close frame 更新下沉为增量 hit-grid mutation，不能在 callback 另建 route authority。

## 待验收

运行 `retained_document_tab_pointer` focused suite，覆盖 root/floating activate、close、measured/min width、resize 与 deterministic route；1k same-frame click 记录 rebuild/alloc/clone count。通过前不进入 `review.md`。
