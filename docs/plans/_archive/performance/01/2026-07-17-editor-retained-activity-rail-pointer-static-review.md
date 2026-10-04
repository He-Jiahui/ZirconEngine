---
related_code:
  - zircon_editor/src/ui/retained_host/activity_rail_pointer
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
reference_sources:
  - dev/godot/editor/editor_node.cpp
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/LevelEditor.cpp
tests:
  - existing retained activity-rail pointer route/layout suites
  - current-source Windows focused Cargo and 1/100/1000-tab generation trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor Retained Activity Rail Pointer 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`zircon_editor/src/ui/retained_host/activity_rail_pointer` 当前共 **23** 个 Rust 文件，已逐文件阅读 **23/23**，覆盖左右 drawer projection、strip/button geometry、route id、surface 构建与 click fallback。动态 Cargo 与 generation/scaling trace 尚未完成，因此继续留在 `pending.md`。

## 主要结论

- pointer event path 只在既有 surface 上 dispatch；没有 per-move/per-click rebuild。local/global fallback 最多做两次小型 hit dispatch，当前不是 MVP 热点。
- `build_host_activity_rail_pointer_layout_with_workbench_layout_frames()` 在每次调用时遍历四个 drawer slots，并为每个 tab 克隆 instance id、把静态 slot 转为新 `String`，构建左右 vectors；`sync()` 只能在这些分配完成后深比较。
- layout 任何差异都会重建 root、左右 strip、全部 button paths/dispatcher/routes。成本随 tool-window tabs 线性，合理的触发条件应是 tool-window/layout generation，而不是任意 slow dirty。
- 本轮不把 `slot` 硬改为 `&'static str`：这只消除小分配，无法解决重复 model projection，并会扩大 route/layout 所有权变更。PERF-MVP-114/EditorUI08 应冻结 immutable activity projection，并让布局、绘制和 pointer surface共享 generation。
- Godot/Unreal 的 editor dock/tab owners 保持长期对象与命令绑定；Zircon 的目标同样是状态/布局变化时更新，而非每个 host slow pass 重建 tab identity。

## 待验收

运行 retained activity-rail focused suites；对 unchanged slow dirty 与一次 tool-window mutation记录 collect/build/rebuild/clone bytes，验证一次 generation build≤1，并覆盖 left/right strip、drawer slot、global/local click route。通过前不进入 `review.md`。
