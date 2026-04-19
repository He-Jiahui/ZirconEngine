---
related_code:
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/apply_gpu_cache_entries.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - docs/assets-and-rendering/render-framework-architecture.md
implementation_files:
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/apply_gpu_cache_entries.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
plan_sources:
  - user: 2026-04-19 继续完成全部的虚拟几何体任务，然后完善 Hybrid GI，不要中途确认
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-cache-entry-residency-cascade.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime_state_ignores_duplicate_gpu_cache_entries_after_first_unique_probe -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI First-Unique GPU Cache Truth

## Goal

把 `Hybrid GI` runtime host 里仍会被 duplicate GPU cache entry 污染的 probe residency truth 收口掉，避免同一个 `probe_id` 在同一份 cache snapshot 里被重复搬槽，并把后续 unique probe 的 cache truth 挤出最终 runtime state。

## Delivered Slice

`apply_gpu_cache_entries(...)` 现在会先按输入顺序对 `(probe_id, slot)` 做 first-unique 去重，再更新：

- GPU-resident probe 集
- stale resident eviction
- resident probe slot truth

这样同一个 probe 的后续 duplicate cache entry 不会再：

- 重写已经确认的 resident slot
- 把 stale resident probe 多次回收/替换
- 阻塞后续 unique probe 进入最终 cache snapshot truth

## Why This Slice Matters

`Hybrid GI` 的 runtime resolve、recent lineage support、GPU prepare 输入都依赖 probe cache residency 作为基础真值。如果 GPU cache snapshot 允许 duplicate probe id 在 host 侧重复搬槽，那么：

- probe cache 主链会重新受输入顺序噪声驱动
- requested-lineage / hierarchy resolve / RT continuation 会建立在错误的 resident probe 集上
- 后续 scene-driven hierarchy gather 与 radiance-cache continuation 都会被错误 probe truth 污染

## Validation Summary

- 红:
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime_state_ignores_duplicate_gpu_cache_entries_after_first_unique_probe -- --nocapture`
- 绿:
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime_state_ignores_duplicate_gpu_cache_entries_after_first_unique_probe -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime -- --nocapture`
  - `cargo check -p zircon_graphics --offline --locked`

## Remaining M5 Todo After This Slice

- `Hybrid GI`: 继续把更完整的 scene-driven hierarchy gather / runtime-source / RT hybrid lighting 主链压进 runtime 与 GPU source。
- `Virtual Geometry`: 继续把 unified indirect / cluster-raster / residency-manager cascade 往更真实的 GPU authority 推进。
