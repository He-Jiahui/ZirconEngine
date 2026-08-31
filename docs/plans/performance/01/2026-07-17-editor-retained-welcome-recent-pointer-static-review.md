---
related_code:
  - zircon_editor/src/ui/retained_host/welcome_recent_pointer
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01/failure-2026-07-17-retained-asset-pointer-full-surface-rebuild.md
reference_sources:
  - dev/godot/editor/project_manager/project_list.cpp
  - dev/slint/internal/core/model/repeater.rs
tests:
  - existing retained Welcome recent pointer open/remove/scroll suites
  - current-source Windows focused Cargo and 10k-row scroll/move storm pending
doc_type: implementation-evidence
status: superseded_by_2026_08_23_current_source_review
---

# Editor Retained Welcome Recent Pointer 逐文件性能静态审查（2026-07-17）

> Superseded by
> `2026-08-23-editor-retained-welcome-recent-typed-item-receipt-hard-cutover-architecture-review.md`.
> Current source has two mirror nodes, no scroll rebuild and O(1) arithmetic row hit; the N-row
> node/rebuild findings below are historical and must not drive current implementation.

## 范围与覆盖

`zircon_editor/src/ui/retained_host/welcome_recent_pointer` 当前共 **20** 个 Rust 文件，已逐文件阅读 **20/20**，覆盖 pane/layout sync、recent row geometry、open/remove route、hover/click/scroll 与 surface rebuild。动态 Cargo、10k-row stress 与 route parity 尚未完成，因此继续留在 `pending.md`。

## 主要结论

- `sync_pane_size()` 正确保留 committed recent paths，layout/state equality fast path也存在；但 scroll 内部绕过该 gate，offset 一变即 full rebuild。
- 每 row 创建 item、open、remove 三个节点，格式化三条 paths、注册 callbacks，并为 Open/Remove route 各 clone project path。ScrollableBox 显式 `virtualization: None`。
- move/click dispatch 将 owned action route转换为 public route；move consumer 只需 hovered index/action，却仍可能 clone path。应采用 state-only move 与 click时才物化 path。
- scroll rebuild 使用旧 hit route更新 hover，未像 menu scroll 一样在新 offset 后重命中；结构修复需同时锁定 hover parity，不能只删除 rebuild。
- Godot project list 与 Slint repeater均把数据 identity 与可见项目 materialization 分开。PERF-MVP-117/EditorUI01 应让 Welcome、asset、hierarchy 和 menu共享 stable row/visible-range input primitives。

## 待验收

覆盖 1/100/10k rows、pane resize、clamp、hover、Open/Remove、missing path 与 deterministic order；1k scroll/move 记录 rebuild、active nodes、path clone bytes、p95。通过前不进入 `review.md`。
