---
related_code:
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/runtime_prepare/execute_runtime_prepare_passes.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/execute.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/collect_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/resident_probe_inputs.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/runtime_prepare/execute_runtime_prepare_passes.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/execute.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/collect_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/resident_probe_inputs.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
plan_sources:
  - user: 2026-04-19 scene-driven screen-probe hierarchy / RT hybrid lighting continuation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-hybrid-gi-recent-lineage-trace-support-continuation.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_trace_lighting_readback_uses_runtime_hierarchy_rt_lighting_after_schedule_clears -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_resolve_render -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_hierarchy -- --nocapture
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Runtime-Resolve To GPU-Prepare RT-Lighting Continuation

## Goal

把 `Hybrid GI` 最近已经落下来的 runtime-side hierarchy RT-lighting continuation，再往下压一层到真实 GPU prepare/source：

- 上一刀已经让 runtime host 在当前 `scheduled_trace_region_ids` 清空后，仍能保住 recent lineage trace support，并继续影响 pending probe 排序与 `build_resolve_runtime()`
- 但 renderer 的 `execute_prepare(...)` 仍然只看当前 frame 的 `scheduled_trace_region_ids`
- 结果是 runtime resolve 虽然已经记住 hierarchy RT-lighting continuation，GPU prepare/readback 却还会在 no-schedule frame 直接掉回全黑 source

这轮目标就是把 `EditorOrRuntimeFrame.hybrid_gi_resolve_runtime` 真正接进 `execute_prepare(...)` 的 resident probe 输入，让 runtime/GPU host 已经确认过的 hierarchy RT-lighting 继续成为 GPU source，而不只停在 post-process resolve。

## Delivered Slice

### 1. 红灯锁定 “runtime resolve 有值，但 GPU prepare 仍然全黑”

新增回归：

- `hybrid_gi_gpu_trace_lighting_readback_uses_runtime_hierarchy_rt_lighting_after_schedule_clears`

测试做了两件事：

- 当前 frame 不再提供 `scheduled_trace_region_ids`
- 只通过 `with_hybrid_gi_resolve_runtime(...)` 给同一个 resident probe 注入 warm / cool 两组 hierarchy RT-lighting continuation

实现前两边 `probe_trace_lighting_rgb` 都是 `[0, 0, 0]`，证明 GPU prepare 当时完全没有消费 runtime continuation source。

### 2. runtime resolve 正式进入 renderer prepare 主链

`execute_runtime_prepare_passes(...)` 现在会把：

- `frame.hybrid_gi_prepare`
- `frame.hybrid_gi_resolve_runtime`

一起传给 `HybridGiGpuResources::execute_prepare(...)`。

随后 `collect_inputs(...)` 会把这份 runtime resolve 继续传到 `resident_probe_inputs(...)`，不再让 renderer prepare 自己只盯当前 trace schedule。

### 3. resident probe GPU 输入新增 runtime hierarchy RT-lighting fallback

`resident_probe_inputs(...)` 现在会先照旧读取当前 frame 的：

- `lineage_trace_support_q`
- `lineage_trace_lighting_rgb`

如果当前 schedule 没有提供 trace source，则会回退到 `HybridGiResolveRuntime`：

- 优先消费 `hierarchy_rt_lighting(probe_id)` 的 RGB + weight
- 再回退到 direct `probe_rt_lighting_rgb`
- 并把 hierarchy RT-lighting weight 或 runtime resolve weight 映射成新的 `lineage_trace_support_q`

这样 no-schedule frame 的 resident probe prepare 输入终于不再是“只有 previous irradiance，没有任何 RT-lighting continuation”，而是开始显式带上 runtime/GPU host 已确认的 hierarchy RT-lighting source。

## Why This Slice Matters

当前 `Hybrid GI` 的 continuity 之前分成两半：

- runtime host / resolve 已经记住 recent hierarchy truth
- GPU prepare / update completion 还只认当前 frame trace schedule

这会让真实 GPU source 在 no-schedule frame 直接掉黑，而 post-process resolve 却还在用 runtime continuation，形成 host-side hierarchy truth 与 GPU-side source truth 的断层。

补上这层之后：

- runtime resolve 不再只是 post-process 私有缓存
- renderer prepare 会把这条 hierarchy RT-lighting continuation 继续压回 resident probe GPU input
- `update_completion.wgsl -> GPU readback -> runtime host -> next frame resolve` 主链开始共享同一份 no-schedule continuation source

这正是后续更完整 `scene-driven screen-probe hierarchy / RT hybrid lighting continuation` 继续往 pending probe、probe gather、真正 scene-driven hierarchy source 收敛前必须补上的中间闭环。

## Validation Summary

- 红灯
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_trace_lighting_readback_uses_runtime_hierarchy_rt_lighting_after_schedule_clears -- --nocapture`
- 绿灯
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_trace_lighting_readback_uses_runtime_hierarchy_rt_lighting_after_schedule_clears -- --nocapture`
- 回归
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_resolve_render -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_hierarchy -- --nocapture`
  - `cargo check -p zircon_graphics --lib --offline --locked`

## Remaining Gaps

- 这轮 continuation 只下沉到了 resident probe GPU prepare/source；pending probe 仍然主要依赖当前 frame trace schedule，尚未拿到更完整的 runtime scene-driven hierarchy continuation source。
- `Hybrid GI` 仍然值得继续推进到更完整的 scene-driven screen-probe hierarchy gather / request / RT hybrid lighting continuation，尤其是把 runtime host 保住的 hierarchy truth 更深地下沉到 pending probe input、probe gather 与真实 radiance-cache update contract。
