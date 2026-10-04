---
related_code:
  - zircon_editor/src/ui/retained_host/shell_pointer
  - zircon_editor/src/tests/host/retained_drawer_resize
  - zircon_editor/src/tests/host/retained_tab_drag
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/SDockingTabStack.cpp
  - dev/godot/editor/editor_node.cpp
tests:
  - immutable lock-free drag-frame source boundary RED then GREEN
  - unchanged resize geometry rebuild boundary RED then GREEN
  - existing drawer-resize/tab-drag route suites and Windows focused Cargo pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor Retained Shell Pointer 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`zircon_editor/src/ui/retained_host/shell_pointer` 当前共 **8** 个 Rust 文件，已逐文件阅读 **8/8**，覆盖 unified input dispatch、drag/drop targets、floating attach/edges、drawer resize capture 与 geometry effects。动态 Cargo、drag storm 与 unchanged-layout counters 尚未完成，因此继续留在 `pending.md`。

## 主要结论与直接修复

- 每次 `update_layout_with_workbench_layout_frames()` 都完整调用 `build_drag_surface()`：重建固定 drag/edge nodes、所有 floating-window attach/edge nodes、formatted paths、route map、dispatcher closures 与 hit/render surface。没有 layout/floating projection equality或 generation gate。
- drag frames 在 surface 创建后不再修改，但旧实现用 `Arc<Mutex<DragTargetFrames>>`，side/document edge move 每次 lock，并保留 poison recovery。已先加入源码 RED 边界，再改为 immutable `Arc<DragTargetFrames>`；闭包仍共享同一帧快照但热路径无锁。
- resize surface为固定 root+3 splitters。旧 `update_resize_surface()` 即使 geometry/state 完全相同也覆写并 full rebuild。`update_target_node()` 现返回真实变化，root/targets 聚合 changed 后才 rebuild。
- `build_drag_surface()` 的 `Arc` 与 closures仍应只在 `{root/layout frames, drawers visible, floating projection generation}` 变化时创建；不应把 cache放在 drag callback 或异步无限队列。
- Unreal Slate docking与 Godot editor docking都维护长期 widget/drag owners；Zircon 可保留 unified typed route/capture，但 layout generation必须与每次 pointer dispatch解耦。

## 待验收

运行 retained drawer-resize 与 tab-drag focused suites，覆盖 document/side/bottom/floating attach/edges、capture until up、native-window bounds；1k move记录 lock=0/p95，重复 identical layout记录 drag/resize rebuild与closure alloc=0。通过前不进入 `review.md`。
