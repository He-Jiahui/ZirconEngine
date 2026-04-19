---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry/extract_registration.rs
  - zircon_graphics/src/runtime/virtual_geometry/declarations/virtual_geometry_runtime_state.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/consume_feedback.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry/extract_registration.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
plan_sources:
  - user: 2026-04-19 把 current + recent confirmed frontier truth 继续压进更深的 completion/page-table/residency 归并点
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_drops_recent_hot_frontier_truth_when_page_leaves_live_extract_before_reappearing -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked page_table -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Live-Extract Frontier Pruning

## Goal

继续把 `current + recent` frontier truth 从 completion/page-table/residency 路径推进到 extract registration 边界，避免 old recent-hot residue 跨过 live extract gap 回流到后续 recycle plan。

## Problem

前两刀已经让 runtime host 把 `current + recent` confirmed frontier truth 写进：

- cooling-frame hysteresis
- feedback completion carry-forward
- GPU completion / page-table apply

但 `register_extract(...)` 还留着一个漏口：

- `current_hot_resident_pages` 会按 `live_page_ids` 裁掉
- `recent_hot_resident_pages` 却不会

结果是，一个 page 即使已经离开当前 live extract：

- resident state 会被裁掉
- hierarchy/page parent 也会被裁掉
- 但 recent-hot residue 仍会留在 runtime cache 里

当同一 page id 之后再次出现在 extract 中时，它会错误地带着旧 hot bias 回来，并影响新的 recycle plan。

## Delivered Slice

### 1. extract registration 现在同步裁剪 recent-hot cache

`register_extract(...)` 现在会像处理 `current_hot_resident_pages` 一样处理：

- `recent_hot_resident_pages`

所以离开 `live_page_ids` 的 page 会同时离开：

- resident slot truth
- current hot truth
- recent hot truth

### 2. frontier carry-forward 现在不会跨 extraction gap 泄漏

这意味着 recent frontier truth 的生命周期终于更接近“当前 runtime live extract”：

- 它可以跨 cooling frame 留下来
- 可以跨 completion/page-table/residency 归并点继续传播
- 但不能跨一个明确的 live extract removal gap 继续残留

### 3. 新增回归覆盖 page leave -> reappear 的 stale bias 漏口

新增红绿回归：

- `virtual_geometry_runtime_state_drops_recent_hot_frontier_truth_when_page_leaves_live_extract_before_reappearing`

它证明：

- page `200` 先成为 recent-hot frontier
- 然后离开 live extract
- 之后又重新出现在 extract 中
- 下一次请求 unrelated page `500` 时，runtime 不会继续把 `200` 当成旧 hot frontier protected page

## Why This Matters

这条修补把 `current + recent` frontier truth 又推进了一层：

- 不只在 completion/page-table/residency 内部保持一致
- 还开始在 extract registration 这类 runtime 边界上保持一致

这样后续更深的 split-merge policy 才不会被 “陈旧 recent-hot residue” 拖回旧状态。

## Validation

- `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_drops_recent_hot_frontier_truth_when_page_leaves_live_extract_before_reappearing -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked page_table -- --nocapture`
- `cargo check -p zircon_graphics --offline --locked`

## Remaining Route

- 继续把这份 `current + recent` frontier truth 压进更深的 runtime merge point，尤其是 still-live runtime state 与后续 recycle / hold / reconnect 合流处。
- 继续补更完整的 split-merge frontier policy，让 confirmed frontier truth 不只在 extract/prepare/complete/apply 这些点上对齐，还能继续约束更深层 replacement / hold / reconnect 级联。
