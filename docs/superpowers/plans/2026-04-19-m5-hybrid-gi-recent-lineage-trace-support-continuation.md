---
related_code:
  - zircon_graphics/src/runtime/hybrid_gi/declarations/hybrid_gi_runtime_state.rs
  - zircon_graphics/src/runtime/hybrid_gi/scene_trace_support.rs
  - zircon_graphics/src/runtime/hybrid_gi/plan_ingestion.rs
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/consume_feedback.rs
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/complete_gpu_updates.rs
  - zircon_graphics/src/runtime/hybrid_gi/prepare_frame/collect_pending_updates.rs
  - zircon_graphics/src/runtime/hybrid_gi/build_resolve_runtime.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
implementation_files:
  - zircon_graphics/src/runtime/hybrid_gi/declarations/hybrid_gi_runtime_state.rs
  - zircon_graphics/src/runtime/hybrid_gi/scene_trace_support.rs
  - zircon_graphics/src/runtime/hybrid_gi/plan_ingestion.rs
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/consume_feedback.rs
  - zircon_graphics/src/runtime/hybrid_gi/pending_completion/complete_gpu_updates.rs
  - zircon_graphics/src/runtime/hybrid_gi/prepare_frame/collect_pending_updates.rs
  - zircon_graphics/src/runtime/hybrid_gi/build_resolve_runtime.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
plan_sources:
  - user: 2026-04-19 scene-driven screen-probe hierarchy / RT hybrid lighting continuation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-hybrid-gi-scene-driven-lineage-trace-support-runtime-and-gpu-source.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime_state_keeps_recent_lineage_trace_support_for_pending_probe_order_after_schedule_clears -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime_state_keeps_recent_lineage_trace_support_in_resolve_runtime_after_schedule_clears -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_resolve_render -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_hierarchy -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_visibility -- --nocapture
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Recent Lineage Trace-Support Continuation

## Goal

继续把 `Hybrid GI` 的 scene-driven screen-probe hierarchy 真值从“只在当前 frame 的 scheduled trace helper 里成立”推进到 runtime host 自己可持续消费的 continuation source：

- 当前 frame 有 trace-support 时，pending probe 排序和 hierarchy resolve weight 都已经会变
- 但一旦下一帧 `scheduled_trace_region_ids` 清空，这条 hierarchy truth 就会立刻掉回 flat path
- 结果是 request priority 会马上退回 root-first，runtime resolve 也会立刻忘掉刚刚由 scene-driven trace hierarchy 建起来的权重差

这轮目标不是发明新的 probe/trace schema，而是让 runtime host 自己记住最近一拍的 lineage trace support，并把它同时喂回 `prepare` 和 `resolve runtime`。

## Delivered Slice

### 1. 红灯锁定 “schedule 一清空就失忆” 的 runtime 漏洞

新增两条 runtime 回归：

- `hybrid_gi_runtime_state_keeps_recent_lineage_trace_support_for_pending_probe_order_after_schedule_clears`
  - 先让 child probe 因为 ancestor-aligned trace support 获得更高 request priority
  - 下一帧把 trace schedule 清空
  - 断言 runtime prepare 仍然会先保 child probe，而不是立刻退回 flat root-first 排序
- `hybrid_gi_runtime_state_keeps_recent_lineage_trace_support_in_resolve_runtime_after_schedule_clears`
  - 先让 hierarchy path 和 flat path 走过相同的 GPU trace-lighting history写回
  - 再把当前 schedule 清空
  - 断言 `build_resolve_runtime()` 仍然会给曾受 scene-driven support 的 hierarchy 路径更高的 resolve weight 与 RT-lighting continuation weight

实现前两条都稳定失败，证明 runtime host 当时并没有保住这条 recent hierarchy truth。

### 2. runtime host 新增 recent lineage trace-support cache

`HybridGiRuntimeState` 现在新增 `recent_lineage_trace_support_q8`：

- `plan_ingestion`
- `consume_feedback`
- `complete_gpu_updates`

都会在更新 `scheduled_trace_regions` 后调用统一的 `refresh_recent_lineage_trace_support()`。

这条 cache 不是永久锁死旧支持，而是：

- 当前 frame 有 scene-driven support 时写入最新值
- 当前 frame 没有 support 时按固定 q8 衰减
- 只在 probe 仍然活着时保留；extract 收缩会同步裁掉 stale entry

因此 runtime host 终于拥有了一条轻量、可衰减、且只依赖自己 scene/runtime 数据的 hierarchy continuation source。

### 3. prepare 排序和 resolve runtime 开始共享同一份 effective support

`collect_pending_updates(...)` 不再只看当前 `scheduled_trace_regions`：

- 现在会优先消费 `effective_lineage_trace_support_score(...)`
- 即 “当前 frame support” 与 “recent decayed support” 的并集真值

`build_resolve_runtime()` 也不再只看当前 frame trace schedule：

- hierarchy resolve weight 会继续吃这条 effective support
- hierarchy RT-lighting continuation 的支持权重也会继续吃它

这样 request、runtime host 和 resolve runtime 终于围绕同一份 recent scene-driven hierarchy support 收敛，而不是每层各记各的“一次性 trace helper”。

## Why This Slice Matters

前一轮已经把 `lineage_trace_support_q + lineage_trace_lighting_rgb` 压进：

- pending update 排序
- GPU probe input
- `update_completion.wgsl`
- GPU readback
- runtime resolve source

但中间仍然断了一拍：

- 当前 frame 一旦没有新的 trace schedule
- runtime host 就会立刻丢掉前一拍刚刚建立起来的 hierarchy support

这会让 `Hybrid GI` 的 scene-driven screen-probe hierarchy 只能在“当前 frame 仍有 trace work”的瞬间成立，无法形成真正的 continuation。

这轮补完后，runtime host 至少已经能把这条 truth 持续到下一拍，并同时影响：

- probe request priority
- hierarchy resolve weight
- hierarchy RT-lighting continuation

为后续更完整的 scene-driven screen-probe hierarchy gather / RT hybrid lighting 路线打通了 runtime-host 侧闭环。

## Validation Summary

- 红灯
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime_state_keeps_recent_lineage_trace_support_for_pending_probe_order_after_schedule_clears -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime_state_keeps_recent_lineage_trace_support_in_resolve_runtime_after_schedule_clears -- --nocapture`
- 绿灯
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime_state_keeps_recent_lineage_trace_support_for_pending_probe_order_after_schedule_clears -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime_state_keeps_recent_lineage_trace_support_in_resolve_runtime_after_schedule_clears -- --nocapture`
- 回归
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_resolve_render -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_hierarchy -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_visibility -- --nocapture`
  - `cargo check -p zircon_graphics --lib --offline --locked`

## Remaining Gaps

- 这轮只把 recent scene-driven hierarchy support 保到 runtime host，并让 `prepare` / `resolve runtime` 共同消费；还没有把这份 continuation 继续编码进 `HybridGiPrepareFrame` 本体，所以下一刀仍然值得继续把它更深地下沉到 GPU input/source contract。
- `Hybrid GI` 仍然缺更完整的 scene-driven screen-probe hierarchy gather / request / RT hybrid lighting continuation，尤其是把 runtime host 保住的 hierarchy truth 继续压到更真实的 GPU-side source，而不只停在 host-side排序和 resolve 权重。
