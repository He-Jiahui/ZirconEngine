---
related_code:
  - zircon_graphics/src/types/mod.rs
  - zircon_graphics/src/runtime/hybrid_gi/prepare_frame.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_resident_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_resident_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
plan_sources:
  - user: 2026-04-17 Hybrid GI should replace runtime default or test-injected irradiance_rgb with real GPU radiance-cache update output
  - user: 2026-04-17 continue the remaining M5 Hybrid GI route without waiting for confirmation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-radiance-cache-lighting-resolve.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - cargo test -p zircon_graphics hybrid_gi_gpu_completion_readback_reports_completed_probe_updates_and_traces --locked
  - cargo test -p zircon_graphics hybrid_gi_gpu_completion_readback_respects_probe_budget_without_evictable_slots --locked
  - cargo test -p zircon_graphics hybrid_gi_gpu_completion_readback_changes_when_previous_irradiance_changes --locked
  - cargo test -p zircon_graphics hybrid_gi_runtime_state_applies_gpu_completed_updates_and_trace_schedule --locked
  - cargo test -p zircon_graphics hybrid_gi --locked
  - cargo test -p zircon_graphics --lib --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Temporal Radiance Cache Update

**Goal:** 把 `Hybrid GI` 的 GPU completion pass 从“只根据本帧 scene/probe metadata 重新合成一份 irradiance”推进到真正会读取上一帧 resident probe radiance-cache history、并按 scheduled trace region 空间分布更新 contribution 的 temporal update baseline。

**Non-Goal:** 本轮仍然不实现真实 traced radiance gather、screen probe gather、surface cache/card atlas、hardware RT lighting 或跨帧 hysteresis/variance estimation。

## Delivered Slice

- `HybridGiPrepareProbe.irradiance_rgb` 不再只是 post-process resolve 的输入，也成为 GPU completion pass 的 history 输入。
- `GpuResidentProbeInput` 新增 `previous_irradiance_rgb`，`execute_prepare(...)` 会把 runtime host 当前 resident probe cache 打包上传进 compute pass。
- `hybrid_gi/shaders/update_completion.wgsl` 现在把 resident probe update 拆成两步：
  - 先基于 quantized probe `position/radius` 与 scheduled trace region 的 `center/radius/coverage` 生成 trace-region-localized contribution，而不是继续依赖 stateless seeded RGB 占位值
  - 再按 `ray_budget + tracing_budget` 生成 temporal weight，把 `previous_irradiance_rgb` 与本帧 contribution 做 deterministic blend
- pending probe completion 仍然走“只输出本帧 contribution”的 bootstrap 路径，不伪装自己已经有历史 radiance-cache；下一帧它们会作为 resident probe 进入同一条 temporal update 路径。
- 当本帧没有 scheduled trace region 时，resident probe 会保持上一帧 history，pending probe 会输出黑值；这让 runtime host 不会在“没有 trace work”的帧里继续合成假 irradiance。

## Runtime And Shader Contract

- runtime host 侧：
  - `HybridGiRuntimeState::build_prepare_frame()` 继续把当前 resident probe cache 导出成 `HybridGiPrepareProbe { irradiance_rgb }`
  - render 完成后，GPU readback 产出的 `probe_irradiance_rgb` 仍然会通过 `complete_gpu_updates(...)` 回写到 runtime host
- renderer 侧：
  - completion pass 现在上传的 resident probe输入包含 `probe_id / slot / ray_budget / spatial_seed / previous_irradiance_rgb`
  - readback contract 仍然保持 `completed_probe_ids / completed_trace_region_ids / probe_irradiance_rgb`，没有把 shader 私有 temporal 状态泄露到 façade
- shader 侧：
  - resident probe path 使用 temporal blend
  - pending probe path 继续输出 bootstrap contribution
  - 这样下一帧 prepare snapshot 会稳定携带真正由 GPU temporal update 生成的 radiance-cache 颜色，而不是继续依赖 runtime 默认值或测试注入色

## Validation Summary

- `hybrid_gi_gpu_completion_readback_reports_completed_probe_updates_and_traces`
  - 证明 resident probe readback 现在与 previous irradiance-aware 期望值一致
- `hybrid_gi_gpu_completion_readback_changes_when_previous_irradiance_changes`
  - 证明在场景 metadata 不变时，只改变上一帧 cache history 也会改变 GPU update 结果，GPU kernel 已经不再是 stateless
- `hybrid_gi_gpu_completion_readback_respects_probe_budget_without_evictable_slots`
  - 证明 temporal update 没有破坏已有 budget/eviction 边界
- `hybrid_gi_runtime_state_applies_gpu_completed_updates_and_trace_schedule`
  - 证明 GPU-produced irradiance 仍会写回 runtime host 并进入下一帧 prepare snapshot

## Remaining Route

- 用真实 ray/screen-probe gather 替换当前 quantized trace-region-localized spatial contribution kernel
- 引入 probe confidence / variance / hysteresis，而不是固定 temporal weight
- 与 screen probe、scene representation、RT hybrid lighting 的更深层联合路径
