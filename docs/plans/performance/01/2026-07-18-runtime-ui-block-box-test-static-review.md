---
related_code:
  - zircon_runtime/src/ui/tests/block_box_layout.rs
  - zircon_runtime/src/ui/layout/pass/taffy_arrange.rs
  - zircon_runtime/src/ui/layout/taffy_bridge
  - zircon_runtime/src/ui/template
  - zircon_runtime/src/ui/v2
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
reference_sources:
  - dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs
tests:
  - 5 BlockBox semantic tests reviewed
  - Taffy native and Zircon fallback pixel parity present
  - v1/v2 container and slot contract parity present
  - persistent-tree slot-probe and current-source Cargo evidence pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI BlockBox测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/block_box_layout.rs` 1/1个tracked Rust文件、309行、5个测试。测试覆盖BlockBox Taffy native选择、Zircon slot-policy fallback、arranged/render/hit一致性，以及v1 template与v2 asset的container/slot contract。

## PERF-MVP-260/261：小树只能锁语义

最大Taffy report只有3个tree nodes，测试没有tree create/insert/style/children/compute或slot probe counter。它可确保persistent Taffy改造保留native/fallback选择与padding/alignment像素，却无法验收当前每容器新建Taffy tree和多consumer线性扫描全局slot Vec。EditorUI02仍需以1/100 nested containers和100/1k/10k nodes证明stable tree create/insert=0、edge slot lookup近O(1)。

## PERF-MVP-276/278：contract与frame ownership

v1/v2测试各自从TOML load/compile/build后以线性control-id/slot scan取节点，属于小fixture正确性门禁；产品compiled generation仍需typed layout/slot DTO，surface不得重parse通用TOML。三个layout测试通过owned`surface_frame()`读取完整report/tree/render/hit，stable frame访问零payload clone仍由EditorUI08/PERF-MVP-278验收。

current-source Cargo、规模counter、MVP workbench BlockBox产品trace和像素完成前，本文件留在`pending.md`，不进入`review.md`。
