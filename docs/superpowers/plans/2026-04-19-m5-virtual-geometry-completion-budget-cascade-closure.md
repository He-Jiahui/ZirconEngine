---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_gpu_uploads_with_slots.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_pending_pages.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/consume_feedback.rs
  - zircon_graphics/src/runtime/render_framework/submit_frame_extract/record_submission/update_virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - docs/assets-and-rendering/render-framework-architecture.md
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_gpu_uploads_with_slots.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_pending_pages.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
plan_sources:
  - user: 2026-04-19 继续完成全部的虚拟几何体任务，不要中途确认，优先推进更深的 residency-manager cascade
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-page-table-confirmed-completion-cascade.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_keeps_processing_later_valid_gpu_completions_after_leading_stale_slot_assignments -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_keeps_processing_later_unique_feedback_completions_after_leading_duplicate_requested_pages -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_gpu -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Completion-Budget Cascade Closure

## Goal

把 `Virtual Geometry` runtime host 里最后两条仍会被 completion 输入顺序污染的 residency cascade 收口掉，避免：

- GPU completion 前部的 stale slot assignment 提前吃掉 `page_budget`
- feedback completion 前部的 duplicate `requested_pages` 提前吃掉 `page_budget`

这两条 residue 都会让同一帧后面的有效 page completion 无法进入最终 page-table / residency truth。

## Delivered Slice

### 1. GPU completion 不再在 slot validation 之前被 `page_budget` 截断

`complete_gpu_uploads_with_replacements(...)` 之前会先对 confirmed GPU assignments 做 `.take(self.page_budget)`，再进入：

- pending 过滤
- replacement 处理
- slot assignability 校验

这样只要前面几个 completion 落在 stale / non-assignable slot 上，后面其实合法的 completion 就会被提前截断。

现在这条提前截断已经移除，runtime completion 会继续遍历后面的 assignment，直到真正处理完输入里的有效 completion。

### 2. Feedback completion 现在先按输入顺序去重，再应用 `page_budget`

`complete_pending_pages(...)` 原来会直接：

- 过滤 `pending_pages`
- `.take(self.page_budget)`

这会让 `requested_pages = [200, 200, 300]` 之类的 feedback 输入把 later unique page 挤出 runtime completion 窗口；更糟的是，duplicate page 在第二次迭代时还可能白白触发一次 eviction，最后既没完成新 page，也把旧 resident page 先踢掉。

现在 feedback completion 改成：

- 仍按输入顺序消费 `requested_pages`
- 先用 seen-set 去重
- 再把 unique pending page 压进本帧 completion budget

这样 duplicate request id 不会再污染 residency cascade。

## Why This Slice Matters

`Virtual Geometry` 当前已经把 page-table / completion / replacement 真值逐步收口到 runtime host，但只要 completion budget 仍然被“无效 assignment”或“重复 request id”提前消耗，runtime host 还是会在关键一跳上退回输入顺序偶然性。

这会直接破坏：

- page-table-confirmed completion cascade
- hot-frontier / split-merge residency 保热
- prepare 下一帧看到的 `available_slots` / `pending_page_requests`

也就是说，前面已经做好的 unified indirect / cluster-raster authority，会被 runtime completion 这一层重新拉回不稳定状态。

## Validation Summary

- 红:
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_keeps_processing_later_valid_gpu_completions_after_leading_stale_slot_assignments -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_keeps_processing_later_unique_feedback_completions_after_leading_duplicate_requested_pages -- --nocapture`
- 绿:
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_keeps_processing_later_valid_gpu_completions_after_leading_stale_slot_assignments -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_keeps_processing_later_unique_feedback_completions_after_leading_duplicate_requested_pages -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_gpu -- --nocapture`
  - `cargo check -p zircon_graphics --offline --locked`

## Remaining M5 Todo After This Slice

- `Virtual Geometry`: 继续把 visibility-owned / GPU-generated indirect authority 往更真实的 GPU args compaction 与 cluster-raster execution ownership 下沉。
- `Virtual Geometry`: 继续补更深的 residency-manager cascade / page-table / completion / split-merge frontier policy。
- `Hybrid GI`: 继续把 scene-driven hierarchy/runtime-source 闭环向更完整的 probe gather / RT hybrid lighting 延伸。
