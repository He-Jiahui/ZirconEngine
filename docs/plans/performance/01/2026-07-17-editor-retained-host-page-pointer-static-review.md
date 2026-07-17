---
related_code:
  - zircon_editor/src/ui/retained_host/host_page_pointer
  - zircon_editor/src/tests/host/retained_host_page_pointer
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
reference_sources:
  - dev/godot/editor/editor_node.cpp
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/LevelEditor.cpp
tests:
  - unused measured-state source boundary RED then GREEN
  - existing retained host-page pointer/overflow suites and Windows focused Cargo pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor Retained Host Page Pointer 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`zircon_editor/src/ui/retained_host/host_page_pointer` 当前共 **20** 个 Rust 文件，已逐文件阅读 **20/20**，覆盖 page projection、text-measured tab allocation、overflow、typed route 与 click dispatch。动态 Cargo、large-page generation trace 与 overflow route parity 尚未完成，因此继续留在 `pending.md`。

## 主要结论与直接修复

- `build_host_page_pointer_layout()` 每次克隆全部 page id/title，查找 active page，测量 visible candidates，并分配 tabs/overflow；`sync()` 只能在这些工作完成后深比较。应由 host-page/title/active/strip-size generation 驱动。
- bridge 旧有 `measured_frames` 会在每次 layout sync resize、在 visible click 写入，但 surface rebuild 和 tab allocator从未读取。点击前已经确认 tab visible，因此 callback fallback frame 与 `layout.tabs.is_empty()` rebuild 分支不可达。
- 已先加入源码边界并确认旧实现为 RED，再删除字段、sync resize、click write、不可达 frame/rebuild。visible click 直接复用 committed `HostPageTabSlot.frame`；hidden page仍用 `route_for_item()`，overflow route不变。
- overflow allocation 使用 shared visible slots 并保持 active page；当前可见 cap 较小，BTreeSet 与二次 visible title width不是主要热点。若未来 cap 提升，应在 generation build 中缓存 width，而非 pointer event 重测。

## 待验收

运行 retained host-page focused suites，覆盖 wide/narrow/active overflow、visible/hidden/overflow click、outer-shell frames；对 unchanged slow dirty 与 page/title mutation记录 clone/text measure/layout/rebuild。通过前不进入 `review.md`。
