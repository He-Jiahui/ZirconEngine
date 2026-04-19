---
related_code:
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/complete_pending_probes.rs
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/consume_feedback.rs
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/complete_gpu_updates.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - docs/assets-and-rendering/render-framework-architecture.md
implementation_files:
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/complete_pending_probes.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
plan_sources:
  - user: 2026-04-19 继续完成全部的虚拟几何体任务，然后完善 Hybrid GI，不要中途确认
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-16-m5-hybrid-gi-feedback-streaming.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime_state_keeps_processing_later_unique_feedback_probe_completions_after_leading_duplicate_requests -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Feedback-Completion Budget Dedup

## Goal

补齐 `Hybrid GI` runtime feedback-completion 路径里仍然残留的一条 probe-budget 顺序污染：duplicate `requested_probe_ids` 不能继续把后面的 unique pending probe completion 截掉。

## Delivered Slice

### 1. `complete_pending_probes(...)` 改成“先去重，再应用 probe budget”

原实现会直接：

- 过滤 `pending_probes`
- `.take(runtime.probe_budget)`

这让 `requested_probe_ids = [200, 200, 300]` 一类 feedback 输入出现时，later unique probe 会被 duplicate probe 挤掉；同时 duplicate probe 的第二次处理还可能白白触发 eviction，导致 runtime host 既失去旧 resident probe，又没把真正的 later pending probe 推成 resident。

现在 `complete_pending_probes(...)` 会：

- 保持 feedback 输入顺序
- 先对 pending probe id 去重
- 再把 unique probe id 填进当前 `probe_budget`

因此 feedback-driven screen-probe residency progression 不再受 duplicate id 污染。

## Why This Slice Matters

`Hybrid GI` 的 scene-driven hierarchy gather / request 主链已经逐步落到 runtime host，但如果 feedback completion 这层还允许 duplicate requested probe 抢走 `probe_budget`，那么：

- 同帧 later unique probe 无法完成 resident promotion
- trace schedule / lineage history 会在错误 probe 集上继续滚动
- 下一帧 prepare / resolve 会拿到被 duplicate request 扭曲过的 probe-cache truth

这会让更上层的 runtime-source / RT hybrid-lighting continuation 在最基础的 cache residency 上重新失真。

## Validation Summary

- 红:
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime_state_keeps_processing_later_unique_feedback_probe_completions_after_leading_duplicate_requests -- --nocapture`
- 绿:
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime_state_keeps_processing_later_unique_feedback_probe_completions_after_leading_duplicate_requests -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu -- --nocapture`
  - `cargo check -p zircon_graphics --offline --locked`

## Remaining M5 Todo After This Slice

- `Hybrid GI`: 继续把 scene-driven hierarchy/runtime-source 闭环往更完整的 screen-probe gather / RT hybrid lighting continuation 推进。
- `Virtual Geometry`: 继续补更深的 unified indirect / cluster-raster / residency-manager cascade。
