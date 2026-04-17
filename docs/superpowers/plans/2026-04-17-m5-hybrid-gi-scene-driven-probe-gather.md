---
related_code:
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/build.rs
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/hybrid_gi_probe_request_sort_key.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
implementation_files:
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/build.rs
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/hybrid_gi_probe_request_sort_key.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
plan_sources:
  - user: 2026-04-17 continue the remaining M5 route without waiting for confirmation
  - user: 2026-04-17 Virtual Geometry still needs residency-manager cascade and Hybrid GI still needs scene-driven radiance cache / probe gather / RT hybrid lighting
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-gpu-completion-source.md
  - docs/superpowers/plans/2026-04-17-m5-hybrid-gi-screen-probe-trace-support-resolve.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - cargo test -p zircon_graphics --offline --locked visibility_context_prioritizes_hybrid_gi_probe_requests_supported_by_scheduled_trace_regions
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_completion_readback_gathers_radiance_from_nearby_resident_probes
  - cargo test -p zircon_graphics --offline --locked hybrid_gi
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Scene-Driven Probe Gather

**Goal:** 把 `Hybrid GI` 再往真正的 scene-driven radiance-cache 路线推进一段，让 probe request scheduling 和 GPU completion 都开始受到当前 scene trace work 与邻近 resident radiance cache 的共同驱动，而不是继续停在 `ray_budget` 排序和纯 trace-region 合成上。

**Non-Goal:** 本轮仍然不引入完整的 screen-probe hierarchy、surface cache、hardware RT gather 或 Lumen-like scene representation。

## Delivered Slice

- `build_hybrid_gi_plan(...)` 现在先确定 `scheduled_trace_regions`，再对 nonresident active probe 做 request 排序。
- 新的 `hybrid_gi_probe_request_sort_key(...)` 不再只看 `ray_budget`：
  - 先计算 probe 对当前 scheduled trace regions 的世界空间支撑度
  - 再用 `ray_budget` 做次级 tie-break
  - 最后用 `probe_id` 保持确定性
- 这意味着 `requested_probe_ids / dirty_requested_probe_ids` 现在会优先选择真正被当前 trace work 支撑的 probe，而不是盲目优先高 budget probe。
- `update_completion.wgsl` 现在把 `resident_probe_inputs` 从“只给 resident temporal update 提供 previous history”推进成真正的 radiance-cache gather source：
  - pending probe completion 会先生成 trace-driven radiance
  - 然后在局部空间范围内 gather 邻近 resident probe 的上一帧 `irradiance_rgb`
  - 最后把 traced radiance 与 gathered resident radiance 做权重融合
- resident probe update 也会使用同一套 gathered resident radiance，作为 traced contribution 的局部 cache support。
- 既有 no-trace 约束保持不变：
  - 没有 scheduled trace region 时，resident probe 继续保留上一帧 history
  - pending probe 仍然输出黑值，不会凭空合成新 radiance

## Why This Slice Exists

- 之前的 resolve 已经能在 post-process 阶段按 scheduled trace region 偏向附近 probe，但 request planning 仍然只按 `ray_budget` 排序，GPU completion 也仍然只消费 trace region + scene light seed。
- 这会造成两段断裂：
  - runtime request 链不能真正优先把 trace work 集中的 probe 拉进 radiance cache
  - GPU completion 不能把已有 resident radiance cache 当作 scene-driven gather source
- 本轮把这两段补上后，当前链路已经形成：
  - `visibility trace support -> requested probes -> resident cache + trace-driven GPU update -> resolve`
- 这样后续继续推进 screen-probe hierarchy / RT hybrid lighting 时，就不需要重新推翻现有 runtime host 与 shader contract。

## Behavior Contract

- probe request priority 现在显式依赖 scheduled trace region support；如果所有 probe support 相同，排序仍然回退到既有的 `ray_budget -> probe_id`。
- resident radiance gather 只在 trace-driven update 路径里参与；它不会在无 trace 帧下单独生成新 radiance。
- gather source 只来自 prepare snapshot 里的 resident probe history，因此 runtime 仍然保持：
  - renderer/GPU 负责生成 radiance-cache 内容
  - host/runtime 只缓存与转发，不伪造 probe lighting

## Validation Summary

- `visibility_context_prioritizes_hybrid_gi_probe_requests_supported_by_scheduled_trace_regions`
  - 证明 scheduled trace region 附近的 probe 会在 request 排序中压过更高 `ray_budget` 但离 trace work 更远的 probe。
- `hybrid_gi_gpu_completion_readback_gathers_radiance_from_nearby_resident_probes`
  - 证明 pending probe completion 现在会被邻近 resident probe 的 radiance-cache 颜色拉偏，而不是只受 trace region 启发式影响。
- `cargo test -p zircon_graphics --offline --locked hybrid_gi`
  - 证明新的 request planning 与 resident gather 没有破坏 Hybrid GI runtime、GPU completion、resolve 与 history 回归。
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明新增 planning helper 与 shader 改动没有留下 crate 编译缺口。

## Remaining Route

- 把当前 trace-supported request + resident gather 继续推进到真正的 scene representation / screen-probe hierarchy
- 让 probe gather 不只依赖 resident cache 邻域，还能接进更高阶的 RT hybrid lighting source
- 把 request priority、radiance update 与未来 probe placement / relocation policy 继续统一到同一条 scene-driven GI 主链上
