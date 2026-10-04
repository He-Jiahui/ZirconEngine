---
related_code:
  - zircon_editor/src/ui/retained_host/asset_pointer
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01/failure-2026-07-17-retained-asset-pointer-full-surface-rebuild.md
  - docs/plans/zircon_editor/editor/09/failure-2026-07-17-editor-asset-catalog-full-rebuild-and-preview-lock.md
reference_sources:
  - dev/slint/internal/core/model/repeater.rs
  - dev/godot/scene/gui/item_list.cpp
  - dev/godot/scene/gui/tree.cpp
tests:
  - existing bridge and retained-host pointer integration suites
  - current-source Windows focused Cargo and 10k-row scroll/move storm pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor Retained Asset Pointer 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`zircon_editor/src/ui/retained_host/asset_pointer` 当前共 **23** 个 Rust 文件，已逐文件阅读 **23/23**：root/common 2、content 6、reference 8、tree 7。动态 Cargo、10k-row scroll/move storm 与 route parity 尚未完成，因此继续留在 `pending.md`。

## 主要结论

- layout constructors 会 clone 全部可见 folder/item/reference id；这只应在 catalog/layout generation 变化时发生。App 层已改为 committed `Arc<AssetWorkspaceSnapshot>` 且 unchanged size 不再调用 constructor。
- 三种 bridge 的 scroll offset 每次变化都执行 `rebuild_surface()`：重新创建 root/viewport/全部 row nodes、格式化 path、注册 dispatcher callback、clone target id、构建 BTreeMap 并 `surface.rebuild()`。
- 三个 `UiScrollableBoxConfig` 的 `virtualization` 都是 `None`；滚动成本与完整 row 数量线性相关。
- `dispatch_event` 为 move/down 都 clone owned target；move consumer 只需要 hovered index/state，仍复制 UUID/folder String。
- sync 的 layout/state equality fast path正确，但不能覆盖 scroll 内部全树 rebuild。

## 责任计划与验收

PERF-MVP-109 已移交 EditorUI01：由统一 pointer dispatch authority 建立 stable row identity、viewport+overscan materialization、增量 scroll transform/hit-grid 与 state-only move dispatch。Editor09 负责提供 generation-owned asset rows，EditorUI08 负责只在对应 surface generation 变化时同步 bridge。验收必须覆盖 1/100/10000 rows、1k scroll/move、list/grid/tree/reference、unknown reference、resize、clamp、click/press/drag 与 deterministic route parity。
