---
related_code:
  - zircon_graphics/src/runtime/hybrid_gi/build_resolve_runtime.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/collect_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/pending_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/resident_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/runtime_trace_source.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu_runtime_source.rs
implementation_files:
  - zircon_graphics/src/runtime/hybrid_gi/build_resolve_runtime.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/collect_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/pending_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/resident_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/runtime_trace_source.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu_runtime_source.rs
plan_sources:
  - user: 2026-04-19 scene-driven screen-probe hierarchy / RT hybrid lighting continuation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-hybrid-gi-runtime-resolve-gpu-prepare-rt-lighting-continuation.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu_runtime_source.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime_state_builds_pending_probe_hierarchy_rt_continuation_after_schedule_clears -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_pending_probe_gpu_trace_lighting_uses_runtime_hierarchy_rt_continuation_after_schedule_clears -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_ -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Pending-Probe Runtime Source Continuation

## Goal

把上一刀只覆盖 resident probe 的 runtime-resolve continuation，继续下沉到 pending probe 的真实 GPU prepare/source：

- `HybridGiResolveRuntime` 之前只稳定覆盖 resident probe
- `pending_probe_inputs(...)` 仍然只认当前 frame `scheduled_trace_region_ids`
- 结果是 trace schedule 清空后，pending probe 仍会在 GPU source 里直接掉回黑值

这轮目标是让 runtime host 已经保住的 hierarchy RT-lighting / resolve-weight truth，真正进入 pending probe 的 GPU input，而不是继续只留在 resident prepare 或 post-process resolve。

## Delivered Slice

### 1. runtime resolve 正式开始跟踪 pending probe

`HybridGiRuntimeState::build_resolve_runtime()` 现在不再只遍历 resident slots，而是会统一收集：

- resident probes
- `pending_probes`
- `pending_updates`

这让 runtime resolve 输出第一次拥有了“pending probe 仍在 hierarchy 主链上”的真实 source 面，而不是把 pending probe 全部丢在 runtime 外面。

### 2. runtime hierarchy trace source 收敛成统一 helper

新增 `runtime_trace_source.rs`，把 runtime fallback 的两条信号收敛成统一 contract：

- hierarchy/direct `rt_lighting_rgb`
- hierarchy resolve weight 映射出的 runtime trace support

`resident_probe_inputs(...)` 和 `pending_probe_inputs(...)` 现在共享同一条 runtime trace-source 逻辑，不再出现 resident/pending 两套 continuation 口径分裂。

### 3. pending probe GPU 输入开始消费 runtime continuation

`collect_inputs(...)` 现在会把 `HybridGiResolveRuntime` 继续传给 `pending_probe_inputs(...)`。

随后 pending probe GPU input 会先读当前 frame schedule：

- `scheduled_trace_support_q`
- `scheduled_trace_lighting_rgb`

如果当前 schedule 不再提供 trace source，则回退到 runtime hierarchy source：

- `lineage_trace_support_q = max(scheduled, runtime)`
- `lineage_trace_lighting_rgb = scheduled != 0 ? scheduled : runtime`

这意味着 no-schedule frame 的 pending probe update 终于不再只能输出 `[0,0,0]`，而会继续沿 runtime-host 已确认的 hierarchy RT-lighting 主链向 GPU source 收敛。

## Why This Slice Matters

之前 `Hybrid GI` 的 continuity 只补齐了一半：

- resident probe 已经能在 no-schedule frame 继续消费 runtime resolve
- pending probe 仍然只盯当前 `scheduled_trace_region_ids`

这会让同一帧里 resident probe 和 pending probe 对 hierarchy continuation 的解释分裂成两套 source truth。补上这层之后：

- runtime host 不再只为 resident resolve 服务
- pending probe 的 GPU prepare/source 也开始共享 runtime hierarchy continuation
- `runtime build_resolve_runtime -> pending_probe_inputs -> update_completion.wgsl -> GPU readback` 终于形成了统一的 no-schedule continuation 主链

这正是继续走向更完整 `scene-driven screen-probe hierarchy gather / request / RT hybrid lighting` 之前必须补上的 runtime/GPU source 闭环。

## Validation Summary

- 红灯
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime_state_builds_pending_probe_hierarchy_rt_continuation_after_schedule_clears -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_pending_probe_gpu_trace_lighting_uses_runtime_hierarchy_rt_continuation_after_schedule_clears -- --nocapture`
- 绿灯
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime_state_builds_pending_probe_hierarchy_rt_continuation_after_schedule_clears -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_pending_probe_gpu_trace_lighting_uses_runtime_hierarchy_rt_continuation_after_schedule_clears -- --nocapture`
- 回归
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_ -- --nocapture`
  - `cargo check -p zircon_graphics --offline --locked`

## Remaining Gaps

- 当前 pending probe continuation 仍主要消费 runtime resolve 已经打包好的 hierarchy truth，还没有把更丰富的 scene-driven probe gather / request state 直接下沉到 GPU source。
- `Hybrid GI` 下一条更自然的主链仍然是更完整的 scene-driven screen-probe hierarchy gather / request / RT hybrid lighting continuation，尤其是把 runtime/visibility 持有的 lineage scene truth 继续压进更深的 probe gather 和 radiance-cache update contract。
