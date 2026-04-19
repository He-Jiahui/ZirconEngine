---
related_code:
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_pending_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_resident_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/runtime_trace_source.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/pending_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/resident_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/tests/hybrid_gi_gpu_runtime_source.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_pending_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_resident_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/runtime_trace_source.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/pending_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/resident_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/tests/hybrid_gi_gpu_runtime_source.rs
plan_sources:
  - user: 2026-04-19 Hybrid GI 的 scene-driven screen-probe hierarchy / RT hybrid lighting continuation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-hybrid-gi-pending-probe-runtime-source-continuation.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_gpu_runtime_source.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_runtime_source -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_ -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Runtime-Hierarchy Irradiance GPU-Prepare Continuation

## Goal

把 `Hybrid GI` 的 runtime-host hierarchy continuation 从 RT-lighting 再推进半步，真正接进 GPU prepare 的 irradiance 合成：

- `HybridGiResolveRuntime` 已经会发布 `hierarchy_irradiance`
- post-process resolve 也已经能消费这条 runtime source
- 但 `update_completion.wgsl` 还没有把它接进 pending/resident probe 的 GPU-side irradiance 合成

结果是当前帧 resident gather 断开时，GPU prepare 仍可能直接掉回黑值，即使 runtime host 已经持有有效的 hierarchy irradiance continuation。

## Delivered Slice

### 1. pending/resident probe input 新增 runtime hierarchy irradiance source

`GpuPendingProbeInput` 和 `GpuResidentProbeInput` 现在都会显式携带：

- `runtime_hierarchy_irradiance_rgb`
- `runtime_hierarchy_irradiance_weight_q`

`pending_probe_inputs(...)` / `resident_probe_inputs(...)` 则通过统一 helper 把 `HybridGiResolveRuntime::hierarchy_irradiance(...)` 量化进 GPU input。

### 2. update_completion.wgsl 开始消费 runtime hierarchy irradiance continuation

shader 新增：

- `apply_runtime_hierarchy_irradiance_continuation(...)`
- `combine_traced_and_gathered_with_runtime_hierarchy_fallback(...)`

契约被固定成：

- 正常 traced/gathered 仍走原有路径
- `traced == 0` 且 runtime hierarchy irradiance 存在时，允许 runtime source 继续接管本次 irradiance 合成
- `traced == 0` 且 runtime source 不存在时，仍保持原来的 “pending 黑值 / resident 保历史” 基线，不额外放开无 trace 的 scene gather

### 3. 新回归直接锁定 runtime hierarchy irradiance GPU continuation

新增 `hybrid_gi_pending_probe_gpu_irradiance_uses_runtime_hierarchy_source_when_scene_gather_is_missing`：

- 当前帧没有 resident gather、没有 scheduled trace
- 只给 runtime resolve 注入 warm/cool 两组 hierarchy irradiance
- 断言 pending probe 的 GPU readback 会跟随 runtime hierarchy irradiance 改变，而不再全部掉回 `[0, 0, 0]`

同时保留 `hybrid_gi_gpu_completion_readback_without_scheduled_trace_regions_keeps_resident_history_and_zeroes_pending_updates`，证明旧 baseline 没被放宽错位。

## Why This Slice Matters

如果 runtime host 已经知道：

- hierarchy resolve weight
- hierarchy irradiance continuation
- hierarchy RT-lighting continuation

但 GPU prepare 只吃后两者里的一半，`Hybrid GI` 就仍然会在 runtime/GPU 边界断开。把 hierarchy irradiance 接进 `update_completion.wgsl` 后：

- runtime host 不再只服务 post-process resolve
- pending/resident probe 的 radiance-cache update 也开始共享这条 continuation
- `scene-driven hierarchy -> runtime resolve -> GPU prepare -> readback` 开始形成更完整闭环

## Validation Summary

- 红灯 -> 绿灯
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_runtime_source -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu -- --nocapture`
- 回归
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_ -- --nocapture`
  - `cargo check -p zircon_graphics --offline --locked`

## Remaining Route

- 这轮仍然属于 runtime/GPU source continuation，不是完整的 screen-probe hierarchy gather/request/RT hybrid lighting 终态。
- 下一条更自然的 M5 主链仍然是把 scene-driven probe hierarchy truth 继续压进更完整的 probe gather、request budgeting 与 RT hybrid-lighting continuation，而不只停在 runtime fallback source。

