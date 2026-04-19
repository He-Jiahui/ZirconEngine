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
tests:
  - zircon_graphics/src/tests/hybrid_gi_gpu_runtime_source.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_pending_probe_gpu_irradiance_uses_requested_lineage_runtime_source_without_trace_schedule -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_pending_probe_gpu_irradiance_inherits_requested_nonresident_ancestor_runtime_source_without_trace_schedule -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_runtime_source -- --nocapture
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_ -- --nocapture
doc_type: milestone-detail
---

# M5 Hybrid GI Requested-Lineage Irradiance Runtime Source

## Goal

把 `Hybrid GI` 的 scene-driven hierarchy continuation 再往 runtime/GPU source 补一刀：

- requested-lineage support 之前已经能保住 resolve weight 和 trace-lighting continuation
- 但在 no-schedule frame 里，如果拿不到新的 hierarchy gather，pending probe 的 runtime irradiance source 仍然可能直接掉回黑值

这轮目标是让 requested-lineage support 在没有当前 trace schedule 的情况下，也能继续托住 runtime irradiance source。

## Delivered Slice

### 1. hierarchy gather 缺失时允许 direct runtime irradiance 回流

`HybridGiRuntimeState::runtime_hierarchy_irradiance(...)` 现在在“没有 farther-resident-ancestor gather”时，不会立刻返回 `None`。

如果满足两件事：

- 当前 probe 自己已经持有上一拍 GPU completion 写回的 `probe_irradiance_rgb`
- `effective_lineage_trace_support_score(...)` 仍然大于零

那么 runtime 会把这份 direct irradiance 重新编码成 `probe_hierarchy_irradiance_rgb_and_weight`。

### 2. fallback 继续沿 nonresident ancestor chain 收集 runtime irradiance

如果当前 probe 自己没有可复用的 irradiance，这条 fallback 现在还会继续沿 `parent_probe_id` chain 查找带有历史 irradiance 的 nonresident ancestor。

只要 requested-lineage / scene-derived support 还在，这些 ancestor radiance 也会继续按层级深度和 ancestor ray budget 被重新编码回 runtime source。

### 3. fallback 只在 nonresident / pending 路径上生效

这条 fallback 只会在 probe 当前不在 `resident_slots` 时启用。

这样 resident probe 不会因为 runtime direct color 被重复喂回而污染正常 hierarchy gather；它只补 pending / no-schedule 这条缺口。

### 4. requested-lineage support 真正进入 irradiance GPU source

新增回归证明：

- flat runtime 仍然输出黑值
- requested-lineage runtime 会把 warm irradiance 继续推回 GPU prepare/readback
- requested child probe 现在还会继续继承 nonresident ancestor 的 runtime irradiance，而不再只支持 probe 自己那一层 fallback

因此 scene-driven requested lineage 已经不再只停在 RT trace-lighting 或 resolve weight，而开始进入 radiance-cache irradiance source 本身。

## Why This Slice Matters

如果 requested-lineage support 只能抬 resolve weight、却不能把 runtime irradiance source 保住，那么 no-schedule frame 里：

- probe gather
- runtime prepare
- GPU update source

仍然会在 irradiance 这一支断开。

这轮把 direct runtime irradiance fallback 接上之后，`request lineage -> runtime host -> GPU prepare` 才开始形成更完整的 scene-driven hierarchy 闭环。

## Validation Summary

- 绿灯
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_pending_probe_gpu_irradiance_uses_requested_lineage_runtime_source_without_trace_schedule -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_pending_probe_gpu_irradiance_inherits_requested_nonresident_ancestor_runtime_source_without_trace_schedule -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_gpu_runtime_source -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked hybrid_gi_ -- --nocapture`

## Remaining Route

- 这轮还只是 requested-lineage 对 runtime irradiance source 的 continuation。
- 下一条更自然的主链仍然是把更完整的 scene-driven hierarchy gather / request / radiance-cache update / RT hybrid lighting 继续收拢到统一 runtime/GPU source contract。
