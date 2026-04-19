---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_gpu_uploads_with_slots.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - docs/assets-and-rendering/render-framework-architecture.md
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_gpu_uploads_with_slots.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
plan_sources:
  - user: 2026-04-19 继续完成全部的虚拟几何体任务，不要中途确认，优先推进更深的 residency-manager cascade
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-page-table-residency-cascade.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_ignores_duplicate_gpu_page_assignments_after_first_unique_completion -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry First-Unique GPU Completion Truth

## Goal

把 `Virtual Geometry` runtime host 里仍会被 duplicate GPU page assignment 污染的 completion truth 收口掉，避免同一个 `page_id` 在同一拍 completion 里被重复搬槽，并把后面的 unique page completion 挤出 residency/page-table 主链。

## Delivered Slice

`complete_gpu_uploads_with_replacements(...)` 现在会先按输入顺序对 GPU `assignments(page_id, slot)` 做 first-unique 去重，再处理：

- explicit replacement truth
- slot assignability
- runtime residency promotion

这样同一个 page 的后续 duplicate completion 不会再：

- 把已经确认的 resident page 重新迁移到另一条 slot truth
- 额外占用当前帧的 eviction / slot authority
- 阻塞后面真正 unique 的 page completion

同一拍 duplicate `replacements(page_id, recycled_page_id)` 也同步改成 first-unique 语义，避免 replacement truth 被后续重复输入重写。

## Why This Slice Matters

`Virtual Geometry` 目前已经把 unified-indirect / cluster-raster / page-table authority 逐层压回 runtime host；如果 GPU completion 自身还允许 duplicate page id 在 host 侧二次搬槽，那么：

- final page-table truth 会重新受输入噪声驱动
- later unique pending page 会被错误挡在 pending queue
- 下一帧 prepare 看到的 `available_slots` / `pending_page_requests` / frontier recycle preference 会继续失真

这会直接破坏更深的 residency-manager cascade。

## Validation Summary

- 红:
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_ignores_duplicate_gpu_page_assignments_after_first_unique_completion -- --nocapture`
- 绿:
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_ignores_duplicate_gpu_page_assignments_after_first_unique_completion -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture`
  - `cargo check -p zircon_graphics --offline --locked`

## Remaining M5 Todo After This Slice

- `Virtual Geometry`: 继续把 visibility-owned truth 往更真实的 GPU-generated args/compaction 与 cluster-raster submission ownership 下沉。
- `Virtual Geometry`: 继续补更深的 residency-manager cascade / page-table / split-merge frontier policy。
- `Hybrid GI`: 继续把 scene-driven hierarchy/runtime-source/RT hybrid lighting 主链补齐。
