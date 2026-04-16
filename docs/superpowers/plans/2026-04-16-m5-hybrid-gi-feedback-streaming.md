---
related_code:
  - zircon_graphics/src/runtime/hybrid_gi.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_feedback.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
  - docs/assets-and-rendering/index.md
implementation_files:
  - zircon_graphics/src/runtime/hybrid_gi.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract.rs
plan_sources:
  - user: 2026-04-16 continue next M5 Hybrid GI slice after runtime-host baseline
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-16-m5-hybrid-gi-runtime-host.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-feedback-streaming.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
  - cargo test -p zircon_graphics hybrid_gi_runtime_state_consumes_feedback_and_promotes_requested_probes --locked
  - cargo test -p zircon_graphics hybrid_gi_runtime_state_leaves_updates_pending_without_evictable_budget --locked
  - cargo test -p zircon_graphics hybrid_gi_runtime_state_tracks_cache_residency_pending_updates_and_trace_schedule --locked
  - cargo test -p zircon_graphics hybrid_gi_visibility --locked
  - cargo test -p zircon_graphics --lib --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M5 Hybrid GI Feedback Streaming Plan

**Goal:** 把此前未消费的 `VisibilityHybridGiFeedback` 接进 `HybridGiRuntimeState`，让 pending probe update 和 trace schedule 在帧后真正推进 viewport probe cache，而不是一直停在前处理 request sink。

**Non-Goal:** 本轮仍然不实现 radiance cache shading、screen probe gather、hardware RT tracing、surface cache/card atlas 或最终的 Lumen-like lighting resolve。

## Delivered Slice

- `HybridGiRuntimeState` 现在记录一个 runtime resident budget，当前由 `RenderHybridGiExtract.probe_budget` 提供，并至少覆盖 extract 基线 resident probe 数量。
- runtime host 新增 `consume_feedback(&VisibilityHybridGiFeedback)`：
  - 把 feedback 的 `scheduled_trace_region_ids` 直接收进 runtime host
  - 只消费当前帧 feedback 中仍处于 pending 的 requested probe
  - resident 数达到 budget 时，只能回收 feedback 提供的 `evictable_probe_ids`
  - 没有可回收 budget 时，request 会继续保持 pending，而不是无上限扩 probe cache
- `WgpuRenderServer::submit_frame_extract(...)` 现在会在 render 完成后调用 `consume_feedback(...)`，再用更新后的 runtime snapshot 写回 façade stats。

## Runtime Rules

- `RenderHybridGiExtract.probe_budget`
  - 当前 CPU fallback baseline 同时充当 requested-probe 预算与 runtime resident-probe 预算
  - 如果 extract 自己已经声明了更多 `resident = true` 的 probe，runtime host 会自动把 budget 抬到至少能容纳这些基线 resident probe
- `consume_feedback(...)`
  - 总是刷新 `scheduled_trace_region_ids`
  - 只会处理 `feedback.requested_probe_ids` 中当前仍处于 `pending` 的 probe
  - 按 feedback 顺序尝试 promote；如果当前 resident 数已经达到 budget，则按 feedback 的 `evictable_probe_ids` 顺序回收 resident probe
  - 无法回收足够 slot 时，本帧剩余 request 会继续留在 pending queue
- 运行结果
  - 下一帧 runtime snapshot 会直接反映 resident/pending/cache-entry 的新规模
  - trace schedule 已经不再只存在于 `VisibilityContext` 的一次性输出，而是会进入 viewport runtime host

## Why This Slice Exists

- Hybrid GI runtime-host baseline 已经能记录 pending update 与 trace schedule，但 `hybrid_gi_feedback` 仍然没有 consumer。
- 如果不补这层，probe cache 只会不断累积 pending update，trace schedule 也不会经历真正的 runtime 消费点，未来 radiance cache / RT lighting 仍然缺一层可替换的宿主边界。
- 这次实现故意保持 CPU-only，并把 budget / eviction policy 固定在 runtime host 内部，好让未来真实 tracing backend 只替换 update completion 来源，而不需要重拆 render-server façade。

## Validation Summary

- `hybrid_gi_runtime_state_consumes_feedback_and_promotes_requested_probes`
  - 证明 feedback request 会在 budget 内消费，并优先复用 evictable resident probe 的 slot
- `hybrid_gi_runtime_state_leaves_updates_pending_without_evictable_budget`
  - 证明在没有可回收 budget 时，runtime host 不会偷偷无限扩 probe cache
- `hybrid_gi_runtime_state_tracks_cache_residency_pending_updates_and_trace_schedule`
  - 证明原有 runtime-host residency/schedule 合同在新反馈消费逻辑下保持兼容
- `hybrid_gi_visibility`
  - 证明前处理的 probe request / dirty request / trace schedule 合同没有被新 consumer 路径破坏

## Remaining Route

- radiance cache lighting resolve / probe gather
- screen probe integration / reservoir update
- RT hybrid lighting backend / BVH-AS coupling
- scene representation / surface cache / card atlas
- 与 history resolve、post-process、Virtual Geometry scene representation 的更深层协同
