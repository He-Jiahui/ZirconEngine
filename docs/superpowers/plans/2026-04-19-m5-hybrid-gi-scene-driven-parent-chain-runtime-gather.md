---
related_code:
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/runtime_trace_source.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/pending_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/resident_probe_inputs.rs
  - zircon_graphics/src/runtime/hybrid_gi/build_resolve_runtime.rs
  - zircon_graphics/src/runtime/hybrid_gi/declarations/hybrid_gi_runtime_state.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu_runtime_source.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/runtime_trace_source.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/pending_probe_inputs.rs
  - zircon_graphics/src/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/resident_probe_inputs.rs
plan_sources:
  - user: 2026-04-19 Hybrid GI 更完整的 scene-driven screen-probe hierarchy / gather / RT hybrid-lighting continuation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_gpu_runtime_source.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_runtime_source -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Hybrid GI Scene-Driven Parent-Chain Runtime Gather

## Goal

把 `Hybrid GI` 的 runtime source continuation 再往前推进一层，不再要求 `HybridGiResolveRuntime` 必须为当前 probe 预先写好 exact hierarchy source，改为允许 encode-side 直接沿当前 extract 的 screen-probe parent chain 做 runtime gather。

## Delivered Slice

### 1. runtime source lookup 现在支持 scene-driven parent-chain gather

`runtime_trace_source(...)` 与 `runtime_irradiance_source(...)` 现在都接受当前 `RenderHybridGiExtract`。

当当前 probe 没有 exact runtime entry，或者 exact entry权重为零时，encode side 会：

- 沿当前 extract 的 `parent_probe_id` chain 向上遍历 ancestor
- 查找 ancestor 的 runtime hierarchy irradiance / hierarchy RT source
- 必要时回退到 ancestor 的 direct `probe_rt_lighting_rgb`
- 按 parent-chain depth 做衰减，再重新量化成 GPU prepare 消费的 `(support_q, packed_rgb)`

### 2. pending / resident probe inputs 统一改走这条 gather contract

`pending_probe_inputs(...)` 与 `resident_probe_inputs(...)` 现在都把 current extract 传进 runtime source gather。

这意味着 runtime source 不再是：

- host 先把 exact probe map 准备好
- encode side 只做 key lookup

而是升级成：

- host 提供 runtime history
- encode side 结合当前 scene hierarchy 再决定怎样把 ancestor history 继续流进当前 probe

### 3. deeper hierarchy continuation 现在真正进入 GPU prepare

新增回归证明：

- 当 pending probe 本身没有 exact runtime source
- 直接 parent 也没有 exact runtime source
- 但更远的 runtime grandparent 仍然带有 hierarchy irradiance / RT continuation

GPU prepare 仍然会通过当前 scene-driven parent chain 把这份 source gather 到当前 pending probe，而不再直接掉回 flat black fallback。

## Why This Matters

之前 `build_resolve_runtime()` 已经能把 requested-lineage / descendant-lineage continuation 重编码回 runtime maps，但最后一段 still 依赖 “exact probe_id 是否在 map 里”。

这会让 hierarchy continuity 仍然主要停留在 host 预编码层，而不是 scene-driven gather 层。

本轮之后：

- 当前 frame 的 probe hierarchy 拓扑本身开始参与 runtime gather
- runtime irradiance / RT continuation 不再只靠 exact child entry 存活
- deeper screen-probe hierarchy 可以直接在 encode-side 延续，而不必每次都回头扩 host 预计算矩阵

这正是继续推进 `screen-probe hierarchy / probe gather / RT hybrid lighting continuation` 所需要的边界。

## Validation Summary

- `hybrid_gi_gpu_runtime_source`
- `hybrid_gi_gpu`
- `cargo check -p zircon_graphics --offline --locked`

新增重点回归：

- `hybrid_gi_pending_probe_gpu_trace_lighting_gathers_runtime_grandparent_hierarchy_source_when_exact_probe_entry_is_missing`
- `hybrid_gi_pending_probe_gpu_irradiance_gathers_runtime_grandparent_hierarchy_source_when_exact_probe_entry_is_missing`

## Remaining Route

- 继续把这条 parent-chain runtime gather 向更完整的 scene-driven probe gather / request / radiance-cache resolve 扩展
- 在 capability 满足时继续往 RT hybrid lighting / deeper screen-probe hierarchy / scene representation 方向推进
- 让 runtime host、GPU prepare、post-process resolve 都消费同一条 scene-driven hierarchy truth，而不再靠多处平行 fallback 维持 continuity
