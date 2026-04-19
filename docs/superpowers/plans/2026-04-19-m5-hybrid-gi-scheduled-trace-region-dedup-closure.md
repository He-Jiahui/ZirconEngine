---
related_code:
  - zircon_graphics/src/runtime/hybrid_gi/scene_trace_support.rs
  - zircon_graphics/src/runtime/hybrid_gi/plan_ingestion.rs
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/consume_feedback.rs
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/complete_gpu_updates.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - docs/assets-and-rendering/render-framework-architecture.md
implementation_files:
  - zircon_graphics/src/runtime/hybrid_gi/scene_trace_support.rs
  - zircon_graphics/src/runtime/hybrid_gi/plan_ingestion.rs
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/consume_feedback.rs
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/complete_gpu_updates.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
plan_sources:
  - user: 2026-04-19 继续完成全部的虚拟几何体任务，然后完善 Hybrid GI，不要中途确认
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-hybrid-gi-scene-driven-lineage-trace-support-runtime-and-gpu-source.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime_state_deduplicates_scheduled_trace_region_ids_before_lineage_support_scoring -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Scheduled Trace-Region Dedup Closure

## Goal

把 `Hybrid GI` runtime host 里会被 duplicate `scheduled_trace_region_ids` 放大的 scene-driven lineage support residue 收口掉，确保 trace schedule 真值不会在 request / resolve / RT continuation 主链上被重复 region id 人为抬高。

## Delivered Slice

`HybridGiRuntimeState` 现在会在三条入口上统一先去重 scheduled trace region ids，再进入 scene-trace support 刷新：

- `ingest_plan(...)`
- `consume_feedback(...)`
- `complete_gpu_updates(...)`

这样 duplicate region id 不会再重复计入：

- `current_lineage_trace_support_score(...)`
- `recent_lineage_trace_support_q8`
- `recent_requested_lineage_support_q8`
- `build_resolve_runtime()` 的 hierarchy resolve / RT continuation 权重

## Why This Slice Matters

`Hybrid GI` 当前已经把 screen-probe hierarchy support 继续压进 runtime host 与 GPU source。如果 trace schedule 自身还允许 duplicate region id 在 host 侧重复加权，那么：

- pending probe 排序会被虚假的 scene support 改写
- recent lineage support 会被错误地延长
- runtime resolve / RT hybrid-lighting continuation 会把重复 schedule 当成真实场景证据

这会让 scene-driven hierarchy 主链重新退回输入噪声驱动。

## Validation Summary

- 红:
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime_state_deduplicates_scheduled_trace_region_ids_before_lineage_support_scoring -- --nocapture`
- 绿:
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime_state_deduplicates_scheduled_trace_region_ids_before_lineage_support_scoring -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu -- --nocapture`
  - `cargo check -p zircon_graphics --offline --locked`

## Remaining M5 Todo After This Slice

- `Hybrid GI`: 继续把 scene-driven screen-probe hierarchy / runtime-source / RT hybrid lighting 闭环往更完整的 probe gather 与 GPU source 收束。
- `Virtual Geometry`: 继续推进更深的 unified indirect / residency-manager cascade。
