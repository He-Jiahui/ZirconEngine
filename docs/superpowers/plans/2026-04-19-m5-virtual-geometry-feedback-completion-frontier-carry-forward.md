---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_pending_pages.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/consume_feedback.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/ordered_evictable_pages_for_target.rs
  - zircon_graphics/src/runtime/virtual_geometry/declarations/virtual_geometry_runtime_state.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_pending_pages.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
plan_sources:
  - user: 2026-04-19 把 current + recent confirmed frontier truth 再继续压进更深的 completion/page-table/residency 归并点
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_feedback_completion_carries_recent_frontier_truth_into_reconnected_ancestor_after_descendant_leaves_residency -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked page_table -- --nocapture
  - cargo test -p zircon_graphics --offline --locked confirmed_virtual_geometry_completion_deduplicates_replacement_truth_after_page_table_normalization -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Feedback Completion Frontier Carry-Forward

## Goal

继续把 `Virtual Geometry` 的 `current + recent confirmed frontier truth` 压进更深的 `completion / page-table / residency` 归并点，避免 feedback completion 分支落后于 GPU completion 分支。

## Problem

上一刀已经把 runtime host 的 hot frontier carry-forward 推进到：

- `recent_hot_resident_pages`
- `ordered_evictable_pages_for_target(...)`
- `complete_gpu_uploads_with_replacements(...)`
- `apply_gpu_page_table_entries(...)`

但还留着一条明显分叉：

- `consume_feedback(...) -> complete_pending_pages(...)`

这条 no-GPU-completion 的 completion 路径此前只会：

- 按 `current + recent` frontier truth 选择先回收谁
- promote requested page 进入 resident set

却不会像 GPU completion 分支那样，把 frontier truth 再写回 newly completed page。

结果是：

- ancestor reconnect 虽然在 completion 当帧得到了正确保护
- 但当 hotter descendant 随后离开 residency 时
- 这个 newly completed ancestor 会立刻掉回冷页
- 下一次 recycle plan 又会优先把它踢掉

## Delivered Slice

### 1. feedback completion 现在和 GPU completion 一样会 carry frontier truth

`complete_pending_pages(...)` 现在会在真正 promote 前先读取：

- `self.page_or_lineage_is_hot(page_id)`

如果 requested page 正处在 `current + recent` confirmed frontier line 上，就会在 promote 之后把它写回 `current_hot_resident_pages`。

### 2. no-GPU-completion 分支不再落后于主 completion 路径

这样 `consume_feedback(...)` 分支就不再只是“借用 hot frontier 做一次 eviction 排序”，而是会和 `complete_gpu_uploads_with_replacements(...)` 一样，把 frontier carry-forward 真值继续压进 residency state。

换句话说，feedback completion 不再只是临时行为层，而是正式参与同一条 runtime truth 链：

- hot frontier input
- completion promotion
- resident state
- cooling-frame carry-forward
- next prepare recycle

### 3. 新增红绿回归证明 descendant 离场后 ancestor 仍被保护

新增回归：

- `virtual_geometry_runtime_state_feedback_completion_carries_recent_frontier_truth_into_reconnected_ancestor_after_descendant_leaves_residency`

它证明：

- ancestor `200` 通过 feedback completion 在 hot descendant `800` 的 frontier 上被重新接回
- 更热的 descendant `800` 之后离开 residency
- 下一次请求 unrelated page `500` 时
- runtime 仍会优先回收 colder unrelated resident `300`
- 不会把刚刚 reconnected 的 ancestor `200` 立刻当成冷页回收

## Why This Matters

这条修补继续把 M5 剩余主链从“hot frontier 影响 recycle 排序”推进到了“hot frontier 影响 feedback-side completion 产出的 resident truth”。

这样 runtime host 的两条 completion 路径终于更接近同一 contract：

- GPU completion 路径
- feedback completion 路径

后续更深的 split-merge policy 才能继续围绕统一的 `completion -> residency -> next prepare` 链路收口，而不是再次被分叉路径拖回浅层修补。

## Validation

- `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_feedback_completion_carries_recent_frontier_truth_into_reconnected_ancestor_after_descendant_leaves_residency -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked page_table -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked confirmed_virtual_geometry_completion_deduplicates_replacement_truth_after_page_table_normalization -- --nocapture`
- `cargo check -p zircon_graphics --offline --locked`

## Remaining Route

- 继续把这份 `current + recent` frontier truth 压进更深的 residency/page-table/recycle 归并点，减少 completion 分支已经收口但 extract/runtime registration 仍有残余入口的情况。
- 继续补更完整的 split-merge frontier policy，让 confirmed frontier truth 不只影响 ancestor reconnect，还能继续主导后续更深层 recycle / hold / replacement 收敛。
