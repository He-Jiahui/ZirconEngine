---
related_code:
  - zircon_editor/src/ui/retained_host/tab_drag
  - zircon_editor/src/tests/host/retained_tab_drag
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/SDockingTabStack.cpp
  - dev/godot/editor/editor_node.cpp
tests:
  - existing retained tab-drag document/drawer/floating/split route suites
  - current-source Windows focused Cargo and 1/100/1000-tab drop profile pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor Retained Tab Drag 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`zircon_editor/src/ui/retained_host/tab_drag` 当前共 **8** 个 Rust 文件，已逐文件阅读 **8/8**；首轮分组输出截断后，`host_resolution.rs` 与 `route_resolution.rs` 已单独完整重读。动态 Cargo 与 large-workspace drop profile 尚未完成，因此继续留在 `pending.md`。

## 主要结论

- pointer move target selection已统一走 `HostShellPointerBridge`；本目录主要处理最终 drop route，不会每 move 重建 surface。
- `precise_drop_target_with_workbench_layout_frames()` 每次构造 `TabStripHitBox`，为目标 drawer/document strip 克隆全部 instance/title/host/workspace path，再分配一个过滤 dragging id 的 vector，并逐 tab调用 runtime text measure 求 midpoint。
- `drop_host_for_tab()` 还会扫描 active drawers、main page document trees、exclusive pages和 floating workspaces寻找当前 host；随后目标选择可能再次扫描 active drawer/page。单次 drop 是 O(layout nodes + target tabs)，且分配集中在 UI 松手帧。
- 当前算法的线性 midpoint 与 deterministic anchor 正确，不适合用 hash iteration 改写。PERF-MVP-119 应在 drag start 复用 document-tab/drawer-header 已提交的 frames和 Workbench generation-owned instance→host index；drop只过滤当前 id并解析 route。
- text width/cache若需要更新，应在 tab title/font/DPI generation变化时完成，不应在 drop 事件临时测量。

## 待验收

覆盖 document/drawer/floating attach与edges、split before/after、same-host no-op、active/fallback page；对1/100/1000 tabs记录 drop p95、clone bytes、text measures和layout node visits。通过前不进入 `review.md`。
