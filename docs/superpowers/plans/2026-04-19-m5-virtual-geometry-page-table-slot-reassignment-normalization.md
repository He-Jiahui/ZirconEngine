---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry/normalized_page_table_entries.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/apply_gpu_page_table_entries.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/record_submission/update_virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry/normalized_page_table_entries.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/apply_gpu_page_table_entries.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/record_submission/update_virtual_geometry_runtime.rs
plan_sources:
  - user: 2026-04-19 Virtual Geometry 剩余主链收敛到更深的 residency-manager cascade / page-table / completion / frontier policy convergence
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/record_submission/update_virtual_geometry_runtime.rs
  - cargo test -p zircon_graphics --offline --locked confirmed_virtual_geometry_completion_deduplicates_replacement_truth_after_page_table_normalization -- --nocapture
  - cargo test -p zircon_graphics --offline --locked confirmed_virtual_geometry_completion_normalizes_reassigned_page_table_truth_before_runtime_apply -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_keeps_reassigned_page_table_owner_in_next_frontier_recycle_plan -- --nocapture
  - cargo test -p zircon_graphics --offline --locked page_table -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hot_frontier_truth -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Page-Table Slot-Reassignment Normalization

## Goal

把 `Virtual Geometry` runtime host 对 GPU page-table readback 的解释进一步收敛成单一 authoritative truth，避免：

- `apply_gpu_page_table_entries(...)`、
- `confirmed_virtual_geometry_completion(...)`、
- 下一帧 `pending_page_requests(...) / ordered_evictable_pages_for_target(...)`

对同一份 raw `page_table_entries` 推导出不同的 resident slot / completion / frontier recycle 结果。

## Problem

此前 runtime host 已经有两条局部收敛：

- duplicate GPU completion 会先做 first-unique 去重；
- duplicate page-table entry 也会阻止 later noise 直接覆盖 resident truth。

但还留着一个更深的缺口：当同一 GPU snapshot 同时出现

- 某个 resident page 的 stale duplicate slot，
- 另一个 pending page 对该 slot 的 takeover，
- 以及原 resident page 随后迁移到新 slot 的 final owner truth，

runtime apply 与 record-side completion confirm 可能会各自保留不同的 page-table 解释，最终导致：

- completion stats 与 runtime page table 不一致，
- moved resident page 在 runtime host 里消失，
- 下一帧 recycle 继续错误地把 hot newly-completed page 当成唯一可回收 slot。

## Delivered Slice

### 1. page-table normalization 抽成 shared helper

新增 `normalized_page_table_entries(...)`，按下面的规则把 raw `page_table_entries` 规范化：

- 从尾到头反向扫描；
- 同时按 `page_id` 与 `slot` 去重；
- 只保留“从最终 snapshot 往回看仍然 surviving”的 page-slot pair；
- 最后按 slot 排序成 deterministic final table。

这比简单的 “forward first-unique” 或 “naive last-writer” 更接近实际 final truth：

- later stale duplicate 不会抹掉一个更早但仍然有效的 resident slot；
- 真正 surviving 的 slot reassignment 仍然会被保留下来。

### 2. runtime apply 与 completion confirm 共用同一份 truth

`apply_gpu_page_table_entries(...)` 与
`confirmed_virtual_geometry_completion(...)` 现在都改为消费 `normalized_page_table_entries(...)`。

结果是：

- runtime resident slot bookkeeping、
- confirmed completion count、
- replacement inference、
- inherited hot-page carry-forward

全部建立在同一份 final table 上，不再出现 “record path 认为 page 还在，runtime host 却把它挤掉” 的分裂。

### 3. frontier recycle 开始跟随 reassigned page-table truth

新增 runtime/frontier regression 证明：

- page `300` 在同一 GPU snapshot 里先出现在旧 slot，再在 `700` take-over 后迁移到 slot `2`；
- runtime host 现在会保留 `300 -> slot 2` 的 final resident truth；
- 下一帧 `500` 的 recycle plan 会正确回收 colder moved page `300`，
  而不是因为 runtime table 丢失 `300` 就去错误回收 hot newly-completed page `700`。

## Why This Matters

这条修补继续把 M5 剩余主链收束到用户当前指定的方向：

- deeper `residency-manager cascade`
- `page-table / completion` truth convergence
- `frontier policy` 不再受 stale slot residue 污染

它没有扩展新的 GPU feature，但把 runtime host 对 page-table truth 的解释变成后续 deeper residency cascade 可以稳定依赖的地基。

## Validation

- `cargo test -p zircon_graphics --offline --locked confirmed_virtual_geometry_completion_normalizes_reassigned_page_table_truth_before_runtime_apply -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked confirmed_virtual_geometry_completion_deduplicates_replacement_truth_after_page_table_normalization -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_keeps_reassigned_page_table_owner_in_next_frontier_recycle_plan -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked page_table -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked hot_frontier_truth -- --nocapture`
- `cargo check -p zircon_graphics --offline --locked`

## Remaining Route

- 继续把这份 normalized `page_table / completion / hot frontier` truth 压进更深的 `residency-manager cascade`，减少 stale lineage / slot residue 在后续 recycle 阶段的残留。
- 继续把 split-merge frontier policy 的更深层保护建立在 confirmed page-table truth 上，而不是建立在单帧 pending queue 或 fallback slot 推断上。
