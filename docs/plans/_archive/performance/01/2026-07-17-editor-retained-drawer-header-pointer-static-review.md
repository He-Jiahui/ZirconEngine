---
related_code:
  - zircon_editor/src/ui/retained_host/drawer_header_pointer
  - zircon_editor/src/tests/host/retained_drawer_header_pointer
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
reference_sources:
  - dev/godot/editor/editor_node.cpp
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/LevelEditor.cpp
tests:
  - measured-frame clone/no-op source boundaries RED then GREEN
  - existing retained drawer-header pointer suites and Windows focused Cargo pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor Retained Drawer Header Pointer 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`zircon_editor/src/ui/retained_host/drawer_header_pointer` 当前共 **21** 个 Rust 文件，已逐文件阅读 **21/21**，覆盖 left/right/bottom surface projection、measured tab frames、route construction 与 click dispatch。动态 Cargo、same-frame click storm 与 generation trace 尚未完成，因此继续留在 `pending.md`。

## 主要结论与直接修复

- layout producer 在每次调用时遍历 drawer stacks、克隆 slot/instance ids、分配 surface/items；只有完整 layout 形成后 `sync()` 才能深比较。该上游成本与 activity rail 同属 tool-window/layout generation 问题。
- click callback 每次调用 `update_measured_frame()`；旧实现即使 host frame 未变化也覆写并 full rebuild root、所有 surfaces/tabs、paths、dispatcher 与 routes。
- 已先增加源码边界并确认旧实现为 RED，再加入 same measured frame no-op。首次测量、width/position 变化与 drawer generation 仍保留 rebuild。
- rebuild 旧实现 `.cloned()` 整个 `Vec<Option<UiFrame>>`，现借用 store 并逐项 `copied().flatten()`；缺失 frame 的 minimum-width fallback 不变。
- 后续 EditorUI08 应复用 activity/drawer immutable projection，并在单 tab geometry 改变时 patch hit node；不得让 activity rail、drawer header、painter各自扫描 tool-window model。

## 待验收

运行 retained drawer-header focused suites，覆盖 left/right/bottom、measured/min width、resize 与 route；1k same-frame click 记录 rebuild/alloc/clone，unchanged slow dirty 记录 layout collect count。通过前不进入 `review.md`。
