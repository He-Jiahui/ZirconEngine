---
related_code:
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/apply_gpu_cache_entries.rs
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/complete_gpu_updates.rs
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/mod.rs
  - zircon_graphics/src/runtime/hybrid_gi/residency_management/clear_pending_update.rs
  - zircon_graphics/src/runtime/hybrid_gi/residency_management/probe_in_slot.rs
  - zircon_graphics/src/runtime/hybrid_gi/residency_management/promote_to_resident.rs
  - zircon_graphics/src/runtime/hybrid_gi/residency_management/promote_to_resident_in_slot.rs
  - zircon_graphics/src/runtime/hybrid_gi/residency_management/reserve_slot.rs
  - zircon_graphics/src/runtime/hybrid_gi/residency_management/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/gpu_completion.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/submit/collect_gpu_completions.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/record_submission/update_hybrid_gi_runtime.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
implementation_files:
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/apply_gpu_cache_entries.rs
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/complete_gpu_updates.rs
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/mod.rs
  - zircon_graphics/src/runtime/hybrid_gi/residency_management/clear_pending_update.rs
  - zircon_graphics/src/runtime/hybrid_gi/residency_management/probe_in_slot.rs
  - zircon_graphics/src/runtime/hybrid_gi/residency_management/promote_to_resident.rs
  - zircon_graphics/src/runtime/hybrid_gi/residency_management/promote_to_resident_in_slot.rs
  - zircon_graphics/src/runtime/hybrid_gi/residency_management/reserve_slot.rs
  - zircon_graphics/src/runtime/hybrid_gi/residency_management/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/gpu_completion.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/submit/collect_gpu_completions.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/record_submission/update_hybrid_gi_runtime.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
plan_sources:
  - user: 2026-04-17 continue M5
  - user: 2026-04-17 Hybrid GI still needs scene-driven radiance cache / probe gather / RT hybrid lighting
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-gpu-completion-source.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-scene-driven-probe-gather.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime_state_applies_gpu_cache_snapshot_as_residency_truth
  - cargo test -p zircon_graphics --offline --locked hybrid_gi
  - cargo test -p zircon_graphics --offline --locked render_server_bridge
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Cache-Entry Residency Cascade

**Goal:** 把 `Hybrid GI` runtime host 从“只消费 `completed_probe_ids`”推进到“把 GPU readback 的 `cache_entries` 当成 radiance-cache residency truth”，让 probe slot、pending update 清理和下一帧 prepare snapshot 跟随 GPU cache snapshot，而不是继续只靠 host 推断。

**Non-Goal:** 本轮仍然不实现完整的 screen-probe hierarchy、surface cache、hardware RT gather 或 Lumen-like RT hybrid lighting。

## Delivered Slice

- `HybridGiGpuCompletion` 现在会保留 renderer readback 的完整 `cache_entries`，而不是只带 `completed_probe_ids / completed_trace_region_ids / probe_irradiance_rgb`。
- `submit_frame_extract/submit/collect_gpu_completions.rs` 会把这份 radiance-cache snapshot 一起带回 server/runtime 记录阶段。
- `HybridGiRuntimeState::apply_gpu_cache_entries(...)` 新增了 GPU cache-truth 同步路径：
  - 当前 host resident probe 里，不在 GPU snapshot 的 probe 会被移除
  - snapshot 里的 `(probe_id, slot)` 会被提升为当前真实 resident ownership
  - 已出现在 GPU snapshot 里的 pending update 会被自动清理
  - `evictable_probes` 会在同步后只保留仍然真实 resident 的 probe
- 为了支撑 GPU 指定 slot，runtime host 新增了 `clear_pending_update / probe_in_slot / reserve_slot / promote_to_resident_in_slot` 这一组 residency-management helper。
- `update_hybrid_gi_runtime(...)` 现在会先应用 `cache_entries`，再消费 `completed_probe_ids / completed_trace_region_ids / probe_irradiance_rgb`。

## Why This Slice Exists

- 之前的 renderer 已经能把 Hybrid GI radiance-cache 的 `cache_entries` 读回，但 runtime host 仍然只根据 `completed_probe_ids` 推断 probe residency。
- 这会留下一个缺口：
  - 如果 GPU completion 在同一帧里重排了 probe slot
  - 或者 cache snapshot 已经把某个 probe 记成 resident，但 `completed_probe_ids` 本身不足以完整表达整张 cache 的最终状态
  - host 的 probe-slot / pending-update / evictable 状态就会滞后于 GPU 真值
- 本轮补上后，Hybrid GI 的 radiance-cache 主链终于形成了 `GPU cache snapshot -> runtime host truth -> next prepare frame` 的闭环。

## Behavior Contract

- GPU readback 到达时，`cache_entries` 的优先级高于 host 侧已有 resident-slot 记录。
- `completed_probe_ids` 仍然保留并继续驱动“哪些 pending probe 本帧完成”，但 authoritative residency/slot 状态来自 GPU cache snapshot。
- 同步后的下一帧 `HybridGiPrepareFrame` 会立刻反映真实 resident probe、剩余 pending update 与 trace schedule。

## Validation Summary

- `hybrid_gi_runtime_state_applies_gpu_cache_snapshot_as_residency_truth`
  - 证明 runtime host 会按 GPU cache snapshot 移除旧 resident probe、提升新 resident probe，并把剩余 pending update 级联到下一帧 prepare snapshot。
- `cargo test -p zircon_graphics --offline --locked hybrid_gi`
  - 证明新的 cache-truth 路径没有破坏 Hybrid GI runtime、GPU completion、resolve 与 scene-driven probe gather 回归。
- `cargo test -p zircon_graphics --offline --locked render_server_bridge`
  - 证明 render-server submit 主链继续保持稳定。
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明 runtime/server completion contract 扩展没有留下 crate 编译缺口。

## Remaining Route

- 更完整的 scene-driven radiance cache / screen-probe hierarchy，而不是继续只同步 slot/residency truth
- 把 probe gather 和 cache-truth 链接到更高阶的 RT hybrid lighting source
- 继续把 request priority、cache residency、probe placement/relocation 收口到统一的 Hybrid GI 主链
