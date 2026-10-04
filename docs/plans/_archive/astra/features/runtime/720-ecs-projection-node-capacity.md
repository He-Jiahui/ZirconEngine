---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/00-ui-architecture-performance-reassessment-2026-09-02.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
related_records:
  - docs/plans/astra/features/runtime/37-runtime-ui-static-contract-alignment.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/717-inline-widget-arrangement-capacity.md
implementation_files:
  - zircon_runtime/src/ui/surface/ecs_projection.rs
tests:
  - zircon_runtime/src/ui/surface/ecs_projection.rs
---

# Runtime ECS UI projection node capacity

The `UiSurface::ui_ecs_projection` hot path already knows the retained tree's
node count, but built the node snapshot with iterator `collect()` and therefore
relied on geometric vector growth. The projection now reserves
`self.tree.nodes.len()` and performs the same one-pass construction with
`push`. Its interaction helper also borrows component flags instead of cloning
them for every node. Node ordering, DTO fields, derived totals, render/hit
counts, and all public snapshot semantics are unchanged.

## Plan completion list

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Runtime11A / RUI-720 | Reserve the retained node-count upper bound and borrow component flags during ECS UI projection. | implemented_pending_validation | RED/GREEN source probes, in-file source guards, scoped Rustfmt/diff checks, and the batched Runtime/Editor contract suites pass; managed Runtime Cargo and Release allocation/latency evidence remain pending. |

## Complexity and allocation boundary

The projection remains `O(nodes + render_commands + hit_entries)` and retains
the existing count maps and DTO ownership. This slice removes geometric growth
of the node output vector and the per-node component-flag clone; it does not
introduce a second tree/index or alter the render/hit aggregation.

## Local evidence

- RED source probes confirmed the exact-capacity and borrowed-flag paths were
  absent before the patch; GREEN probes and in-file source guards confirm the
  retained-node bound, explicit push loop, and borrowed component flags.
- The focused hierarchy/editor contract batch passed `57/57` after the
  adjacent Editor test-contract repair; the final broader non-tooling batch
  passed `1320/1320` across 343 modules in `94.188s`; the subsequent Runtime723
  follow-up reran the same loader and passed `1320/1320` in `5.353s`.
- Rustfmt, `git diff --check`, and optimization-record whitespace checks pass.
- The current combined non-tooling Runtime/Editor batch covers 347 modules and
  passes `1331/1331` tests in `6.361s`.
- These are source/contract checks only and do not claim product CPU,
  allocator, RSS, or p50/p95/p99 acceptance.

## Managed acceptance gate

This slice joins the existing asynchronous Runtime/Editor admission recorded
in `696`; no new coordinator request or status query was issued. The row stays
`implemented_pending_validation` until an owner-attributed Windows Release run
measures ECS projection allocation counts and latency at the declared UI-node
workloads.
