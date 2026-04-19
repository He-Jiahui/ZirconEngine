---
related_code:
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/runtime_trace_source.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/pending_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/resident_probe_inputs.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu_runtime_source.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/runtime_trace_source.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/pending_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/resident_probe_inputs.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu_runtime_source.rs
plan_sources:
  - user: 2026-04-19 scene-driven screen-probe hierarchy / RT hybrid lighting continuation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-hybrid-gi-pending-probe-runtime-source-continuation.md
  - docs/superpowers/plans/2026-04-19-m5-hybrid-gi-requested-lineage-rt-runtime-source.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_gpu_runtime_source.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_pending_probe_gpu_trace_lighting_blends_runtime_hierarchy_source_with_current_trace_schedule -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_runtime_source -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_ -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Scheduled + Runtime Trace Source Blending

## Goal

继续把 `Hybrid GI` 的 scene-driven screen-probe hierarchy / RT hybrid-lighting 主链往下收口。

前几刀已经让 runtime host 能在 no-schedule frame 保住：

- pending probe 的 hierarchy RT runtime source
- requested-lineage nonresident ancestor 的 RT runtime source

但 `pending_probe_inputs(...)` / `resident_probe_inputs(...)` 仍然保留一条过早折平的旧逻辑：

- 只要当前 frame `scheduled_trace_lighting_rgb != 0`
- runtime hierarchy RT source 就会被完全覆盖

这意味着一旦当前 frame 还有 trace schedule，runtime host 已经确认的 hierarchy RT continuation 又会在 GPU source 入口被抹掉，scene-driven hierarchy truth 依然没有真正闭环。

## Delivered Slice

### 1. 红灯锁定 “有 current trace schedule 时 runtime hierarchy continuation 仍被覆盖”

新增回归：

- `hybrid_gi_pending_probe_gpu_trace_lighting_blends_runtime_hierarchy_source_with_current_trace_schedule`

测试构造的是：

- probe `200` 是 pending child
- probe `100` 是 nonresident parent
- 当前 frame 仍有 scheduled trace region，且 local trace tint 是固定灰色
- runtime host 分别持有 warm / cool 两种 ancestor RT history

实现前两条路径都会输出同一个 `[96, 96, 96]`，证明 current schedule 仍然在 GPU input 侧把 runtime hierarchy RT continuation 完全盖掉。

### 2. scheduled/runtime trace source 现在统一走 merge contract

`runtime_trace_source.rs` 新增：

- `merge_trace_sources(...)`

它会把两类 source 收束成同一份 GPU input contract：

- 当前 frame `scheduled_trace_support_q + scheduled_trace_lighting_rgb`
- runtime host `runtime_trace_support_q + runtime_trace_lighting_rgb`

不再使用“scheduled 非零就直接覆盖 runtime”的二选一逻辑。

### 3. pending / resident probe 统一改为 blend，而不是单边覆盖

`pending_probe_inputs(...)` 与 `resident_probe_inputs(...)` 现在都改成共享同一条 merge 逻辑：

- `lineage_trace_support_q` 继续保留两边 support 的较强值
- `lineage_trace_lighting_rgb` 在 scheduled/runtime 同时存在时按 support 做加权混合

这让 runtime hierarchy RT continuation 在 current schedule 仍存在时也能继续进入 `lineage_trace_lighting_rgb`，而不再只在 no-schedule frame 才生效。

### 4. hierarchy truth 继续沿同一份 GPU source 主链下沉

这轮没有新增 encode-side 或 shader-side 私有旁路。

仍然沿原有主链推进：

- `build_resolve_runtime()`
- `runtime_trace_source(...)`
- `pending_probe_inputs(...) / resident_probe_inputs(...)`
- `update_completion.wgsl`
- GPU readback

因此 current-schedule frame 与 no-schedule frame 现在终于开始共享同一份 runtime-host hierarchy truth，而不是在 GPU source 入口重新分叉。

## Why This Slice Matters

如果 runtime continuation 只能在 “没有 current trace schedule” 时生效，那么 `Hybrid GI` 仍然没有真正完成 scene-driven hierarchy 闭环：

- runtime host 记得 hierarchy lineage truth
- 当前 frame 只要还有 local trace work，GPU source 就重新退回 flat scheduled tint

补上这层之后，runtime hierarchy RT continuation 才真正开始覆盖两种情况：

- no-schedule continuation
- current-schedule + runtime-history blending

后续继续推进更完整的 screen-probe hierarchy gather / request / probe gather / RT hybrid-lighting continuation 时，就可以继续沿这条统一的 runtime/GPU source contract 往下压，而不需要重新解释“当前 frame local trace”和“runtime hierarchy history”之间的 ownership。

## Validation Summary

- 红灯
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_pending_probe_gpu_trace_lighting_blends_runtime_hierarchy_source_with_current_trace_schedule -- --nocapture`
- 绿灯
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_pending_probe_gpu_trace_lighting_blends_runtime_hierarchy_source_with_current_trace_schedule -- --nocapture`
- 回归
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_runtime_source -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_ -- --nocapture`
  - `cargo check -p zircon_graphics --offline --locked`

## Remaining Route

- `Hybrid GI` 还可以继续往下推进更完整的 scene-driven screen-probe hierarchy gather / request / probe gather / RT hybrid-lighting continuation，但当前 “runtime source 只在 no-schedule 生效” 这条折平断层已经补上。
- `Virtual Geometry` 的下一条更自然主链仍然是更深的 visibility-owned unified indirect / GPU-generated args / cluster-raster submission ownership，以及更深的 residency-manager cascade。
