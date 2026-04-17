---
related_code:
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_pending_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_resident_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_trace_region_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/hybrid_gi_completion_params.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/seed_quantization.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/mod.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_pending_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_resident_probe_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/gpu_trace_region_input.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/hybrid_gi_completion_params.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
plan_sources:
  - user: 2026-04-17 continue the remaining M5 Hybrid GI route without waiting for confirmation
  - user: 2026-04-17 replace runtime default or test-injected irradiance_rgb with real GPU radiance-cache update output
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-gpu-completion-source.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-temporal-radiance-cache-update.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - cargo test -p zircon_graphics hybrid_gi_gpu_completion_readback_without_scheduled_trace_regions_keeps_resident_history_and_zeroes_pending_updates --locked
  - cargo test -p zircon_graphics hybrid_gi_gpu_completion_readback_concentrates_radiance_on_probes_near_scheduled_trace_regions --locked
  - cargo test -p zircon_graphics hybrid_gi_gpu_completion_readback_reports_completed_probe_updates_and_traces --locked
  - cargo test -p zircon_graphics hybrid_gi_gpu_completion_readback_respects_probe_budget_without_evictable_slots --locked
  - cargo test -p zircon_graphics hybrid_gi_gpu --locked
  - cargo test -p zircon_graphics hybrid_gi --locked
  - cargo test -p zircon_graphics --lib --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M5 Hybrid GI Traced Radiance Source Uplift

**Goal:** 把 `Hybrid GI` 的 GPU completion source 从“deterministic seeded RGB placeholder”推进到真正受 probe/trace 空间关系驱动的 radiance update baseline。

**Non-Goal:** 本轮仍然不实现真实硬件 RT gather、screen-probe gather、surface cache/card atlas、scene card representation 或 probe confidence/hysteresis。

## Delivered Slice

- `execute_prepare(...)` 现在不再只上传 `probe_id / slot / ray_budget` 一类占位输入，而是把 resident/pending probe 的 quantized `position/radius` 和 scheduled trace region 的 quantized `center/radius/coverage` 一起送进 compute pass。
- `update_completion.wgsl` 现在按 probe 与 trace region 的空间距离生成 radiance contribution：
  - 只消费 `min(trace_region_count, tracing_budget)` 条 active trace region
  - probe 离 trace region 越近，得到的 contribution 越大
  - contribution 会按 trace region coverage、probe ray budget 与 tracing budget 共同调节强度
- resident probe path 继续执行 temporal update，但现在 blend 的是“trace-region-localized contribution + previous irradiance history”，而不是 seeded RGB。
- pending probe path 在有 active trace region 时输出空间化 contribution；没有任何 scheduled trace region 时则输出黑值，不再伪造新的 radiance-cache 颜色。
- no-trace 帧的 resident probe 会保持上一帧 irradiance history，不会因为 completion pass 被调用就无条件改写缓存。

## Runtime And Shader Contract

- runtime host 仍然只暴露 `HybridGiPrepareFrame` 和 `HybridGiGpuReadback` 这组 renderer-local 合同，没有把 `wgpu` 或 shader 私有结构泄漏到 `RenderServer` façade。
- `HybridGiPrepareProbe.irradiance_rgb` 继续承担 resident radiance history 输入；GPU readback 的 `probe_irradiance_rgb` 继续承担下一帧 cache history 输出。
- shader 现在把“有没有 active trace work”作为显式分支条件：
  - resident probe：`contribution == 0` 时保留 previous history
  - pending probe：`contribution == 0` 时保持黑值
- 这条规则把 runtime host 的“等待真实 GPU radiance”语义固定下来，避免任何无 trace work 帧仍然产生伪造 GI 更新。

## Validation Summary

- `hybrid_gi_gpu_completion_readback_without_scheduled_trace_regions_keeps_resident_history_and_zeroes_pending_updates`
  - 证明无 scheduled trace region 时，resident probe 保持历史值，pending probe 不再合成新 radiance
- `hybrid_gi_gpu_completion_readback_concentrates_radiance_on_probes_near_scheduled_trace_regions`
  - 证明 trace-region-localized kernel 会让靠近 trace region 的 probe 获得显著更高的 irradiance
- `hybrid_gi_gpu_completion_readback_reports_completed_probe_updates_and_traces`
  - 证明新的空间化 kernel 没有破坏已有 completion id / trace completion / irradiance readback 合同
- `hybrid_gi_gpu_completion_readback_respects_probe_budget_without_evictable_slots`
  - 证明 probe budget / eviction 边界仍然独立生效
- `cargo test -p zircon_graphics hybrid_gi --locked`
  - 证明 runtime host、GPU completion、lighting resolve 三条 Hybrid GI 子链在新 kernel 下保持一致
- `cargo test -p zircon_graphics --lib --locked` 与 `validate-matrix.ps1 -Package zircon_graphics`
  - 证明本轮 uplift 没有破坏 `zircon_graphics` 其余 M4/M5 能力族

## Remaining Route

- 把当前 quantized trace-region-localized spatial heuristic 替换成真实 traced radiance gather 或 screen-probe gather
- 引入 probe confidence / variance / hysteresis，而不是固定 temporal blend weight
- 把 radiance update completion 与更高阶 scene representation、RT hybrid lighting、surface cache/card atlas 连接起来
