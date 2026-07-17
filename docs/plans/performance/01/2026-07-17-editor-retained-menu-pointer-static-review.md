---
related_code:
  - zircon_editor/src/ui/retained_host/menu_pointer
  - zircon_editor/src/tests/host/retained_menu_pointer
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01/failure-2026-07-17-retained-asset-pointer-full-surface-rebuild.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
reference_sources:
  - dev/slint/internal/core/menus.rs
  - dev/godot/scene/gui/popup_menu.cpp
tests:
  - item-tree borrowing/linear route/no-op source boundaries RED then GREEN
  - existing retained menu pointer nested/scroll/dispatch suites and Windows focused Cargo pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor Retained Menu Pointer 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`zircon_editor/src/ui/retained_host/menu_pointer` 当前共 **26** 个 Rust 文件，已逐文件阅读 **26/26**，覆盖 chrome/layout projection、menu tree、popup geometry、root/submenu route、hover/click/scroll state 与 surface 构建。动态 Cargo、large-menu build 与 1k-scroll trace 尚未完成，因此继续留在 `pending.md`。

## 主要结论与直接修复

- layout builder 每次调用都会投影菜单树、测量 popup rows、克隆 preset names，并通过 UI asset 构建 menu slot frames；必须由 menu/chrome/preset/size generation gate，不能在任意 slow dirty 中重复执行。
- 旧 `menu_items_for_layout()` 即使 `layout.menus` 已持有 committed tree 仍整树 clone；`popup_grid_layout()` 为取 `.len()` 也 clone，rebuild 又 clone root 和每层 children。现改为 `Cow<[MenuItemSpec]>`：正常路径全程借用，只有 legacy/default fallback 构造 owned rows。
- 旧 surface insertion 为每个 visible row 调用两次 `menu_item_route_index(root, path)`，每次从 root preorder 扫描并递归跳过子树，flat N rows 达 O(N²)。现每层只定位一次起点，并按 `menu_item_subtree_len` 线性推进稳定 preorder id。
- scroll 不再 clone 包含 path/action String 的 owned route；popup clamp 只计算一次 metrics；already-closed state 不再重建空 menu surface。
- root popup scroll 与 submenu topology 变化仍 full rebuild tree/dispatcher/path/route，ScrollableBox `virtualization: None`。该结构性 P0 已并入 PERF-MVP-112/EditorUI01；不能用 debounce 隐藏 O(N)。

Slint 的 menu shadow tree 由 property tracker 仅在 dirty 时重建并按 id cache；Godot PopupMenu 保留 item/shape/cache 并在 scroll container 上做命中。Zircon 应继续使用 typed route/preorder identity，但将 committed menu tree、visible popup rows与 scroll transform 分离。

## 待验收

运行 `retained_menu_pointer` focused suite，覆盖 root/nested popup、disabled rows、action id、multi-column、scroll/hover recompute、dismiss 与 deterministic route；构造 1k/10k rows/presets 记录 build scaling、clone bytes、active nodes、1k-scroll rebuild/p95。通过前不进入 `review.md`。
