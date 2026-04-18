---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/slot_is_assignable.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_gpu_uploads_with_slots.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/complete_pending_pages.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/slot_is_assignable.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
plan_sources:
  - user: 2026-04-18 Virtual Geometry 的 deeper unified indirect / cluster raster / residency-manager cascade
  - user: 2026-04-18 继续列出所有剩余 todo，作为 tasks，然后继续深入
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-merge-back-child-hysteresis.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-visibility-owned-lineage-segments.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_rejects_gpu_slot_recycling_when_current_evictable_set_withdraws_page
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Current Evictable Slot Recycling Guard

**Goal:** 把 `Virtual Geometry` runtime residency-manager 的 slot recycling 再收紧一层，确保 GPU completion 只信当前帧的 `evictable_pages` 真值，而不会继续因为 runtime 内部旧状态把已经被 visibility/hysteresis 撤回保护的 page 错误回收。

**Non-Goal:** 本轮不重写 uploader shader、page table format 或真正的 GPU-owned residency manager。

## Delivered Slice

- `slot_is_assignable(...)` 不再把 `self.evictable_pages` 当成 slot recycling 的兜底真值。
- 现在 slot 复用只允许以下情况：
  - slot 已经属于目标 `page_id`
  - 当前调用显式传入的 `evictable_pages` 仍包含占位 resident page
  - slot 本身是 free/future slot
- 这让 `complete_gpu_uploads_with_slots(...)` 与当前帧 visibility-owned/hysteresis-owned evictable 集保持一致，不会再被 ingest-plan 阶段遗留下来的旧 `self.evictable_pages` 误导。

## Why This Slice Exists

- 当前 M5 路线已经把 `merge-back child hold`、`multi-level frontier collapse hold` 等保护逻辑前推到 visibility/hysteresis 层。
- 但 runtime residency completion 之前仍然会把历史 `self.evictable_pages` 当成 slot 可复用依据之一。
- 这会导致一个真实风险：
  - 当前帧已经撤回某个 page 的 `evictable` 资格
  - 但 runtime 仍可能因为旧状态允许新 page 直接抢占它的 slot
- 这会破坏本轮已经固定下来的 visibility-owned residency cascade 边界。

## Validation Summary

- `virtual_geometry_runtime_state_rejects_gpu_slot_recycling_when_current_evictable_set_withdraws_page`
  - 证明 GPU completion 现在不会再因为 stale runtime evictable state 抢占已被当前帧撤回保护的 resident slot。
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime`
  - 证明现有 page-table truth、free-slot preference、page snapshot apply、visibility-owned segment prepare 等 runtime regressions 没有回归。
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明 residency-manager guard 的收紧没有引入新的 compile closure 问题。

## Remaining Route

- 把当前“只信当前帧 evictable truth”的 guard 继续推进到更深的 residency-manager cascade，例如 page hierarchy aware slot reuse、shared slot / page ownership 下的更稳 split-merge policy。
- 继续推进 visibility-owned unified indirect authority 下沉到更真实的 GPU submission / cluster raster / readback truth。
