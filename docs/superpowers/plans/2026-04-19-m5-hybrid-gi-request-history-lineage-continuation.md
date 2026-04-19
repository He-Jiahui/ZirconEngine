---
related_code:
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/build.rs
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/ordering/hybrid_gi_probe_request_sort_key.rs
  - zircon_graphics/src/visibility/declarations/visibility_history_snapshot.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
implementation_files:
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/build.rs
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/ordering/hybrid_gi_probe_request_sort_key.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
plan_sources:
  - user: 2026-04-19 scene-driven screen-probe hierarchy / RT hybrid lighting continuation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-hybrid-gi-recent-lineage-trace-support-continuation.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
  - cargo test -p zircon_graphics --offline --locked visibility_context_prefers_previously_requested_hybrid_gi_lineage_when_trace_schedule_clears -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_visibility -- --nocapture
doc_type: milestone-detail
---

# M5 Hybrid GI Request-History Lineage Continuation

## Goal

把 `Hybrid GI` 的 scene-driven screen-probe hierarchy continuation 再往 visibility/request 层补一刀：

- 当前 frame 的 `scheduled_trace_regions` 已经能影响 descendant request
- 但 schedule 清空后，request tie-break 仍然可能立刻跳回无关 lineage
- runtime host 虽然已经开始保 recent trace-support，但 visibility/request 自己还缺一条“上一拍已经确认的 lineage ownership”延续

这轮目标是让 `previous.history_snapshot.hybrid_gi_requested_probes` 成为 request tie-break 的弱 continuation source，而不是把 request ownership 完全交给当前 frame 的瞬时 trace helper。

## Delivered Slice

### 1. previous requested lineage 进入 request tie-break

`hybrid_gi_probe_request_sort_key(...)` 现在会在当前 frame trace-support 已经打平之后，再用 `previous_requested_probe_ids` 做弱 boost。

它的优先级被固定成：

- 先看当前 frame scene-driven trace-support
- 再看 visible hierarchy / depth / stable ordering
- 最后才让上一拍已经请求过的 lineage 继续保住一拍 ownership

这样历史 request 只做 continuation，不会反向覆盖更强的 live scene signal。

### 2. visibility plan 现在能跨 schedule-clears 保住 lineage ownership

`build_hybrid_gi_plan(...)` 现在会显式构造 `previous_requested_probe_ids`，并把它同时传给：

- per-group probe request sort
- interleaved request round sort

因此 “上一帧已经请求过、这一帧 current schedule 暂时清空” 的 descendant lineage 不会立刻在 request 层掉回无关 probe。

### 3. 新回归锁定 visibility/request continuation

新增 `visibility_context_prefers_previously_requested_hybrid_gi_lineage_when_trace_schedule_clears`：

- 先构造一帧 request history
- 下一帧把 trace schedule 清空
- 断言 request 顺序仍然会优先保住上一拍确认过的 lineage，而不是立刻回退到 flat probe-id tie-break

## Why This Slice Matters

如果 runtime host 记住了 recent hierarchy support，但 visibility/request 当帧就把 lineage ownership 丢掉，后续：

- `requested_probe_ids`
- `dirty_requested_probe_ids`
- pending update queue

仍然会抖回无关 probe，scene-driven hierarchy 主链就会在 request 层断开。

这轮让 request history 成为弱 continuation source 后，`visibility -> runtime host -> prepare -> resolve` 才开始共享同一条 lineage continuity，而不是只在 runtime 之后才试图补救。

## Validation Summary

- 绿灯
  - `cargo test -p zircon_graphics --offline --locked visibility_context_prefers_previously_requested_hybrid_gi_lineage_when_trace_schedule_clears -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_visibility -- --nocapture`

## Remaining Route

- 这轮还只是 request/history 层 continuation，下一条更自然的主链仍然是把同一份 scene-driven lineage truth 继续压进更深的 runtime/GPU gather/source contract。

