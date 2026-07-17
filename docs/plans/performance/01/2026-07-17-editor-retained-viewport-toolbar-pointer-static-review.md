---
related_code:
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer
  - zircon_editor/src/tests/host/retained_viewport_toolbar_pointer
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-viewport-toolbar-surface-rebuild-storm.md
reference_sources:
  - dev/godot/editor/scene/3d/node_3d_editor_plugin.cpp
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/SLevelViewportToolBar.cpp
tests:
  - A-to-B-to-A hit preservation behavior regression RED then implementation
  - clicked-control upsert/no-op source boundary RED then GREEN
  - existing retained viewport-toolbar pointer suites and Windows focused Cargo pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor Retained Viewport Toolbar Pointer 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`zircon_editor/src/ui/retained_host/viewport_toolbar_pointer` 当前共 **31** 个 Rust 文件，已逐文件阅读 **31/31**，覆盖 surface layout、arranged-frame projection、control route classification、click dispatch 与 hit surface rebuild。动态 Cargo、same-control click storm、dock/floating route parity 尚未完成，因此继续留在 `pending.md`。

## 主要结论与直接修复

- `sync_surface_frame()` 扫描完整 arranged tree，筛选已知 control、克隆 action keys/frames，任何差异都 full rebuild 所有 viewport toolbar surface。上层每次 slow recompute 还会生成/clone surface frame；结构性 generation cache 归 PERF-MVP-113/EditorUI08。
- 旧 `handle_click()` 无条件用 `vec![clicked_control]` 替换该 surface 的 committed controls 后 full rebuild。除每次点击重建外，A→B 后 A 会从 pointer surface 丢失，直到下一次完整 frame sync。
- 已先增加 A→B 后按点命中 A 的行为回归，以及增量 upsert 源码边界；旧源码为 RED。实现按 `action_key` 查找：相同 frame 返回 false，不重建；frame 改变只替换该项；新 control append；其他 control 保留。
- layout sync 的有效 surface key 集合改为借用 `&str`，避免每次克隆所有 key 后仅用于 `retain`。
- `route_for_control()` 构造 owned `surface_key` 与静态 tool/mode strings；当前只在 frame generation 或实际 route materialization 时可接受，不能回到每 pointer move 重建。

## 待验收

运行 `retained_viewport_toolbar_pointer` focused suite，覆盖 fallback rect、surface projection、A→B→A、dock/floating、tool/snap/toggle/play routes；1k same-frame click 记录 rebuild/alloc，unchanged slow recompute 记录 temporary surface/frame/presentation clone。通过前不进入 `review.md`。
