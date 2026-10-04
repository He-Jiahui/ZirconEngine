---
status: implemented_pending_validation
review_date: 2026-09-05
parent_plan: docs/plans/astra/performance/01-bounded-hotpaths.md
owners:
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/tree/hit_test/geometry_patch.rs
  - zircon_runtime/src/ui/tree/hit_test/cell_membership_patch.rs
  - zircon_runtime/src/ui/surface/frame_hit_test.rs
  - zircon_runtime/src/ui/surface/frame_hit_test/tests.rs
---

# Hit Grid Cell Batching

## Scope

Geometry publication currently removes and reinserts every changed entry separately. When several
changed entries share a dense cell, each update scans or shifts the same unrelated memberships and
repeats persistent copy-on-write work. Base and projected hit indexes must instead stage their
preflighted membership deltas and update each touched cell once while preserving their existing
ordering contracts.

Radius-query admission and fallback semantics are outside this plan. No radius query behavior or
public interface changes are authorized here.

## Milestones

### M0 - Freeze invariants

Preserve exact hit stack, top-hit, route-node, painter-order, projection-order, no-op, and preflight
atomicity behavior. Reuse the current persistent sequence and profiling owners.

### M1 - Batch cell membership updates

Group removals and additions by cell after the complete update batch passes preflight. Rebuild each
touched membership sequence once, retaining unaffected entries and merging additions in the
owner-specific stable order. Base and projected indexes share the batching primitive and retain
their current entry publication rules.

### M2 - Qualification

Focused tests cover no-op, one-entry, and 64-entry geometry changes against cells with 1,000 and
10,000 entries. They assert exact rebuilt-index parity, one membership pass per touched cell,
bounded item visits, and unchanged state after failed preflight. An ignored same-machine release
test checks repeated output parity before recording alternating legacy-model and batched-model
p50/p95/p99 samples for both retained shared snapshots and unique-owner repeated moves. Cargo and release
execution remain a later managed validation stage.

## Acceptance

- A no-op patch touches no cells and performs no membership visits or persistent cell mutation.
- Each changed cell is staged and published at most once per patch batch. Source membership
  cardinality is reported separately from replacement buffer and materialized membership counts;
  it is not a total comparison, copy, or allocation count. The Arc COW counter names only Arc
  copy-on-write clones. Elapsed evidence includes delta/set/Arc allocation and comparison costs.
- Membership work is proportional to existing memberships plus staged deltas per touched cell,
  rather than existing memberships multiplied by changed-entry count.
- Base and projected indexes match a full rebuild for cell membership order, hit stack, top hit,
  and path while preserving their stable entry-index contracts.
- Any preflight failure leaves entries, cell memberships, reverse footprints, and generations
  unchanged, including a valid input-route update followed by invalid geometry admission.
- Same-machine release evidence reports p50/p95/p99 for 1,000 and 10,000 dense memberships; normal
  one-entry sparse P95 does not regress beyond the existing five-percent Astra gate in either
  ownership mode, and K=64 dense P95 must be at most 80% of its paired legacy baseline. Any failed
  gate, including noisy samples, keeps performance qualification pending.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1/M2 | Base 与 projected hit-grid 的按 cell 批量 membership patch、预检原子性、密集 1k/10k 回归 | implemented_pending_validation | 2026-09-09 | 当前源码已使用 `UiCellMembershipPatches::stage/apply` 汇总每个 cell 的 removals/additions，并在统一预检后一次发布；`dense_batches_match_expected_membership_for_1k_and_10k_entries`、`dense_geometry_batches_match_full_rebuild_and_visit_each_cell_once`、失败原子性与 painter reorder 回归均已落源。UI/RG 静态合同批次 `18/18` 通过，测试导入修复后的焦点复跑为 `11/11`；焦点文件 `rustfmt --check` 与 `git diff --check` 通过；忽略的 Windows release paired p50/p95/p99 仍待受管执行。外部 `E:/Git/zr_vm` 最新 `87c112d27f25e2a10e2c078d61ef638954d0f8eb` 仍有 112 tracked/submodule + 88 untracked = 200，阻止不可变 Cargo/release 验收，详见异步日志。 |
