---
related_code:
  - zircon_runtime/src/ui/tests/canvas_slot_layout.rs
  - zircon_runtime/src/ui/tests/canvas_slot_template.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/surface/arranged_tree.rs
  - zircon_runtime/src/ui/template
  - zircon_runtime/src/ui/v2
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
tests:
  - 6 Canvas slot/layout/template tests reviewed
  - stretch anchor z-order layer and v1/v2 contract parity present
  - slot/layer/index scale counters and current-source Cargo pending
  - product canvas/popup trace and pixel/hit parity pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI Canvas slot测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/canvas_slot_layout.rs`与`canvas_slot_template.rs`，共2/2个tracked Rust文件、383行、6个测试。测试覆盖Free/Canvas anchor stretch、pivot/offset、z-order layer分组、hidden render、arranged/render/hit一致性，以及v1/v2 slot contract。

## PERF-MVP-260/277：小fixture没有索引证据

最大fixture只有5个nodes和4个slots。它能锁定same-z layer grouping与draw order，却没有slot entries visited、parent children probes、layer sort/allocation或arranged get计数。生产`build_arranged_tree`仍为slot summary/z-order多次扫描全局slots，Canvas还按parent children做线性contains，后续`UiArrangedTree::get`按Vec线性find；规模化Canvas/popup/workbench overlay可能退化为O(N²)。EditorUI02需以edge slot+dense node index单次DFS构建layer/draw authority。

## PERF-MVP-276：v1/v2 contract只是语义门禁

两个template测试从TOML load/compile/build后检查首slot，证明typed layout/slot cutover必须保留anchor_max/offset/order语义；它们没有parse/map clone/generation计数。EditorUI05/02仍需让compiled generation拥有validated typed canvas contract，surface build TOML parse=0。

## 验收要求

1/100/1k/10k canvas children与slots、same-z 1/100 layers记录slot/children probes、sort、layer/draw build、arranged get、allocation与CPU p95；build近O(N+S+NlogN)，get近O(1)，stable generation rebuild=0。current-source Cargo、MVP popup/canvas产品trace、像素与hit parity完成前，2/2留在`pending.md`。
