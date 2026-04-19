---
related_code:
  - zircon_graphics/src/runtime/hybrid_gi/build_resolve_runtime.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu_runtime_source.rs
implementation_files:
  - zircon_graphics/src/runtime/hybrid_gi/build_resolve_runtime.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu_runtime_source.rs
plan_sources:
  - user: 2026-04-19 scene-driven screen-probe hierarchy / RT hybrid lighting continuation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-hybrid-gi-requested-lineage-irradiance-runtime-source.md
  - docs/superpowers/plans/2026-04-19-m5-hybrid-gi-runtime-resolve-gpu-prepare-rt-lighting-continuation.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_gpu_runtime_source.rs
  - cargo test -p zircon_graphics --offline --locked trace_lighting_inherits_requested_nonresident_ancestor -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_runtime_source -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_ -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Requested-Lineage RT Runtime Source

## Goal

把 `Hybrid GI` 的 requested-lineage no-schedule continuation 再往下补一刀：

- 上一轮已经让 requested-lineage support 能保住 pending probe 的 runtime irradiance source
- 更早一轮也已经让 runtime-resolve 的 hierarchy RT-lighting 真正进入 GPU prepare/source
- 但 `build_resolve_runtime()` 自己在 `runtime_hierarchy_rt_lighting(...)` 里仍然只会消费 resident ancestor gather

结果是，当 pending/nonresident probe 当前拿不到 resident hierarchy RT gather，只剩 probe 自己或 nonresident ancestor 保留的 runtime RT history 时，requested-lineage path 仍然会在 runtime source 这里直接掉回黑值。

这轮目标就是让 requested-lineage support 不只托住 irradiance，也继续托住 RT hybrid-lighting runtime source。

## Delivered Slice

### 1. 红灯锁定 “requested lineage 仍然丢失 ancestor RT runtime source”

新增回归：

- `hybrid_gi_pending_probe_gpu_trace_lighting_inherits_requested_nonresident_ancestor_runtime_source_without_trace_schedule`

测试构造的是：

- probe `200` 是 pending child
- probe `100` 是 nonresident parent
- 当前 frame 没有 trace schedule
- runtime 只保留 parent 的上一拍 GPU RT-lighting history

实现前无论 requested 还是 flat runtime，GPU readback 都是 `[0, 0, 0]`，证明 runtime source 当时还不会把 nonresident ancestor 的 RT history 重新编码回 pending child 的 hierarchy RT source。

### 2. `runtime_hierarchy_rt_lighting(...)` 现在具备 direct-lineage fallback

`HybridGiRuntimeState::runtime_hierarchy_rt_lighting(...)` 现在在 resident ancestor gather 为零时，不再直接返回 `None`。它会转而调用：

- `direct_lineage_rt_lighting_fallback(probe_id)`

这条 fallback 与 irradiance 路径保持同样的 requested-lineage 语义：

- 只在当前 probe 不在 `resident_slots` 时启用
- 先尝试 probe 自己已有的 `probe_rt_lighting_rgb`
- 再沿 `parent_probe_id` chain 继续查找带历史 RT-lighting 的 ancestor

### 3. fallback 不只看 lineage 支持，也继续乘上 budget / intensity

这条 RT fallback 没有把 ancestor history 当成裸色值直接塞回去。新的 runtime support 会继续消费：

- `effective_lineage_trace_support_score(probe_id)`
- probe / ancestor 自己的 `ray_budget`
- `runtime_rgb_intensity(...)`
- `ANCESTOR_TRACE_INHERITANCE_FALLOFF`

因此 runtime host 现在保存的是更接近真实 hierarchy RT continuation 的 runtime source，而不是单纯“只要有旧颜色就回灌”。

### 4. 同一份 runtime source 直接进入 GPU prepare

因为 `runtime_trace_source(...)` 本来就优先消费：

- `hierarchy_rt_lighting(probe_id)`

所以这轮不需要再新增 encode-side 私有旁路。只要 `build_resolve_runtime()` 把 requested-lineage ancestor RT history 重新编码进 `probe_hierarchy_rt_lighting_rgb_and_weight`，pending probe 的 GPU prepare/readback 就会继续沿同一条 runtime-source contract 吃到这份 ancestor RT continuation。

## Why This Slice Matters

如果 requested-lineage support 只能保住：

- resolve weight
- irradiance runtime source

但保不住 RT runtime source，那么 `Hybrid GI` 在 no-schedule frame 里还是会出现一条断层：

- runtime host 还记得 lineage truth
- GPU prepare/readback 的 RT source 却已经掉黑

补上这层之后，requested-lineage support 才真正开始同时驱动：

- runtime resolve weight
- irradiance runtime source
- RT runtime source

这样后续继续推进 screen-probe hierarchy gather / request / radiance-cache update / RT hybrid lighting continuation 时，就能继续沿同一份 scene-driven runtime-source contract 往下收拢，而不是在 RT 支路重新分叉。

## Validation Summary

- 红灯
  - `cargo test -p zircon_graphics --offline --locked trace_lighting_inherits_requested_nonresident_ancestor -- --nocapture`
- 绿灯
  - `cargo test -p zircon_graphics --offline --locked trace_lighting_inherits_requested_nonresident_ancestor -- --nocapture`
- 回归
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_runtime_source -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_ -- --nocapture`
  - `cargo check -p zircon_graphics --offline --locked`

## Remaining Route

- `Hybrid GI` 下一条更自然的主链仍然是把 scene-driven screen-probe hierarchy gather / request / runtime host / GPU source 继续收拢成更完整闭环，而不只停在 no-schedule continuation。
- `Virtual Geometry` 仍然还有更深的 unified indirect / cluster-raster submission ownership 与 residency-manager cascade 要继续推进。
