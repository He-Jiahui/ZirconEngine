---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/ordered_evictable_pages_for_target.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/ordered_evictable_pages_for_target.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
plan_sources:
  - user: 2026-04-19 切回 deeper residency-manager cascade / split-merge frontier policy
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-hot-descendant-frontier-cascade.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_keeps_deepest_hot_descendant_resident_when_same_frontier_has_multiple_hot_descendants -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture
doc_type: milestone-detail
---

# M5 Virtual Geometry Deepest Hot-Descendant Frontier Cascade

## Goal

把 split-merge frontier 的 hot protection 继续推进一层：当同一 reconnecting lineage 上有多个 resident descendants 都仍然 hot 时，runtime 应该优先保留最深的 refined frontier page，而不是继续沿用 “更远 descendant 先回收” 的默认距离启发式。

## Delivered Slice

### 1. hot descendant distance ordering 现在与普通 descendant 相反

`ordered_evictable_pages_for_target(...)` 过去对 descendants 的默认策略是 farther-distance first，这对冷页合理，但对 hot frontier page 不成立。  

现在排序在 `relation == Descendant && hot_resident_pages.contains(page)` 时会改成：

- shallower hot descendant 更早进入 evict 候选
- deepest hot descendant 最后才会被触碰

这样 runtime 在 reconnect 缺失 ancestor page 时，会把 “更精细、仍然 hot 的 branch” 保留下来。

### 2. 同一 frontier 上多个 hot descendants 现在会保 deepest page

新的 regression `virtual_geometry_runtime_state_keeps_deepest_hot_descendant_resident_when_same_frontier_has_multiple_hot_descendants` 锁定了这条 contract：

- resident root + shallower descendant + deeper descendant
- 中间 ancestor page 丢失，需要重新 promote
- 两个 descendants 都仍然 hot

现在系统会回收 shallower hot descendant，把 slot 让给需要 reconnect 的 ancestor，同时继续保留 deepest hot descendant。

## Why This Slice Matters

上一刀只证明了 “hot farther descendant 可以压过 colder nearer descendant”。  
但真正的 split-merge frontier 经常会出现“同一条线上的多个 descendants 仍然同时 hot”的情况。如果这时 runtime 仍然沿用 farther-distance first，那么它会先踢掉最精细的当前 frontier page，等 ancestor 补回后又立刻重新请求同一 deeper branch。

这条 cascade 收口后，runtime residency 开始真正把 `hot frontier` 理解为“当前 refined truth”，而不是只把它当成一个简单的冷/热 tie-break。

## Validation Summary

- focused red/green
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_keeps_deepest_hot_descendant_resident_when_same_frontier_has_multiple_hot_descendants -- --nocapture`
- broader regressions
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture`
