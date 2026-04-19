---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/apply_gpu_page_table_entries.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - docs/assets-and-rendering/render-framework-architecture.md
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/apply_gpu_page_table_entries.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
plan_sources:
  - user: 2026-04-19 继续完成全部的虚拟几何体任务，不要中途确认，优先推进更深的 residency-manager cascade
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-page-table-residency-cascade.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_ignores_duplicate_gpu_page_table_entries_after_first_unique_page -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry First-Unique Page-Table Truth

## Goal

把 `Virtual Geometry` runtime host 接收最终 GPU `page_table_entries` 时仍会被 duplicate `page_id` 污染的落表真值收口掉，避免同一个 page 在同一份 final snapshot 里被重复搬槽，并把后面的 unique resident page 挤出最终 residency truth。

## Delivered Slice

`apply_gpu_page_table_entries(...)` 现在会先按输入顺序对 `(page_id, slot)` 做 first-unique 去重，再更新：

- GPU resident page 集
- stale resident eviction
- final resident slot truth
- inherited hot-page continuation

这样同一个 page 的后续 duplicate page-table entry 不会再：

- 迁移已经确认的 final resident slot
- 反向驱逐后面的 unique resident page
- 把 runtime host 再次拉回“按输入噪声落表”的状态

## Why This Slice Matters

前面已经补上的 `GPU completion` first-unique truth 只是把 raw completion 主链稳定下来；如果最终 `page_table_entries` 仍允许 duplicate page id 重写 resident slot，那么 runtime host 还是会在最后一跳重新失真。

这会直接污染：

- next-frame `available_slots`
- `pending_page_requests` 的 assigned-slot / recycle preference
- hot-frontier / descendant hold 的 residency 真值

所以这条 slice 是更深 residency-manager cascade 的必要收口。

## Validation Summary

- 红:
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_ignores_duplicate_gpu_page_table_entries_after_first_unique_page -- --nocapture`
- 绿:
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_ignores_duplicate_gpu_page_table_entries_after_first_unique_page -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture`
  - `cargo check -p zircon_graphics --offline --locked`

## Remaining M5 Todo After This Slice

- `Virtual Geometry`: 继续把 unified indirect / cluster-raster submission ownership 往更真实的 GPU-generated args source 下沉。
- `Virtual Geometry`: 继续补更深的 split-merge frontier / residency-manager cascade。
- `Hybrid GI`: 继续推进 scene-driven screen-probe hierarchy / RT hybrid lighting continuation。
