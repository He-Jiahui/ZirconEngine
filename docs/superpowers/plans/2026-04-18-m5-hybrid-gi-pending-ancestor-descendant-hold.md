---
related_code:
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/build.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
implementation_files:
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/build.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
plan_sources:
  - user: 2026-04-18 Hybrid GI 的 resolve/runtime-host 侧更完整 scene-driven hierarchy 闭环
  - user: 2026-04-18 继续 M5
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-hybrid-gi-merge-back-child-probe-hysteresis.md
  - docs/superpowers/plans/2026-04-18-m5-hybrid-gi-descendant-request-frontier.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
  - cargo test -p zircon_graphics --offline --locked visibility_context_keeps_resident_hybrid_gi_descendant_probe_hot_while_ancestor_request_remains_pending
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_visibility
doc_type: milestone-detail
---

# M5 Hybrid GI Pending-Ancestor Descendant Hold

**Goal:** 把 `Hybrid GI` 的 screen-probe hierarchy cache policy 再推一层，让 nonresident ancestor probe 仍在 `requested_probe_ids` 时，更深层仍 resident 的 descendant probe 不会在下一帧就掉进 `evictable_probe_ids`。

**Non-Goal:** 本轮不改写 resolve shader payload、GPU completion struct 或 radiance-cache 数据格式。

## Delivered Slice

- `build_hybrid_gi_plan(...)` 现在会根据当前 `requested_probe_ids` 反推出：
  - `requested_frontier_probe_ids`
  - 每条 pending hierarchy request 所挂靠的 active frontier
- 对这些 pending frontier，planning 会继续保护：
  - 当前仍 resident
  - 当前不在 active frontier
  - 但位于 pending-request frontier 下
  - 且已经是该子树最深 resident hidden descendant
  的 probe，不让它进入 `evictable_probe_ids`。
- 这条 hold 会同时覆盖：
  - 直接 child probe merge-back 之后，child 自己的 child request 仍在 pending 的场景
  - 更深层 descendant 在 nonresident 中间 ancestor 还没回来前的 cache-hot continuation

## Why This Slice Exists

- 之前 `Hybrid GI` 只做到了 merge-back 当帧的一拍 hold。
- 当 hierarchy request 挂到第二帧时，系统虽然还会保留 `requested_probe_ids`，但会把 still-resident deeper descendant probe 提前放回 `evictable_probe_ids`。
- 这会让 runtime host 的 probe cache policy 早于 hierarchy request 完成就开始冷却 descendant hot set，导致 scene-driven hierarchy 到 runtime-host 边界再次断裂。

## Validation Summary

- `visibility_context_keeps_resident_hybrid_gi_descendant_probe_hot_while_ancestor_request_remains_pending`
  - 证明 nonresident 中间 ancestor probe 仍在 `requested_probe_ids` 时，更深层 still-resident descendant probe 不会在第二个 collapsed frame 被重新标记成 evictable。
- `visibility_context_holds_resident_hybrid_gi_child_probe_one_frame_when_frontier_merges_back_to_parent`
  - 旧回归已更新到新的、更宽 cache policy：当 descendant request 仍在 pending 时，resident child probe 也会继续保热。
- `hybrid_gi_visibility`
  - 证明 descendant request frontier、lineage-fair budgeting、newly-resident hysteresis 等已有行为没有回归。

## Remaining Route

- 把这层 pending-request hold 继续接到 runtime host 的 probe cache eviction / prepare snapshot 权威上，而不只停在 visibility plan。
- 继续把同一条 hierarchy truth 接进 resolve-side lineage weighting / RT hybrid lighting continuation，让 runtime-host 与 resolve 更一致。
