---
title: Runtime accessibility root projection capacity
category: zircon_runtime
report_id: Runtime838-accessibility-root-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime838 · accessibility root projection capacity

## Scope

The Runtime accessibility snapshot builder already owns the authored root list
and knows its exact length, but the published root projection still started a
zero-capacity `Vec<UiNodeId>`. Large accessibility snapshots therefore paid
geometric growth and copy costs before publication.

## Implementation

- Reserve `surface.tree.roots.len()` for the published root vector before the
  existing deadline, visibility, relation-target, and budget checks.
- Continue pushing only admitted roots in the existing tree order; filtered,
  hidden, and missing roots remain excluded.
- Keep empty root lists zero-capacity and leave the existing Runtime709
  resolution scratch reuse untouched.
- Add a lower source/order regression and the ignored
  `RUNTIME838_ACCESSIBILITY_ROOT_CAPACITY_BENCH_V1` Release marker.

No accessibility role, relation, diagnostic, focus, budget, node-map, or
publication semantics changed. Tooling production remains out of scope.

## Deterministic work model

For 4,096 authored roots, the old zero-capacity collector models 11 geometric
growth events while the exact root-list bound starts full (`11→0`). An empty
root list remains zero-capacity. This is allocation-shape evidence only; it is
not a claim about allocator, CPU, RSS, or product p50/p95/p99 performance.

## TDD and local evidence

- The Python source/model contract was intentionally RED against the original
  zero-capacity collector, then GREEN after the bounded reservation and test
  module wiring (`2/2`).
- The lower Rust source regression preserves root order, exact non-empty
  capacity, and empty-path behavior; the ignored Release marker reports
  p50/p95/p99 fields and compares the legacy and reserved growth models.
- Exact-file Rustfmt and Python compilation pass. The focused thirteen-contract
  Runtime/Editor batch passes `49/49` tests; the refreshed one-process
  non-tooling loader covers `861` modules and passes `3489/3489` tests in
  `33.065s`, with zero failures, errors, load errors, or skips.
- Managed Cargo/Windows Release, allocator, and accessibility product
  percentile evidence remain pending.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/accessibility/extract.rs` | `F87AA27D67184EE005C45B9DB130D92E8FAAD44DCA701ECD7D6404ECA03F23B0` |
| `zircon_runtime/src/ui/accessibility/extract/root_capacity_tests.rs` | `D45B712C84C78DA199709CE7BC3C30015DB929858F6C85B51D7F26F0DDE86FCA` |
| `tools/tests/test_runtime_accessibility_root_capacity_performance_contract.py` | `1E5F85B6CD9276E339459DA2CD8BFA203D9FD4658BD14D948E7AA1A883173C43` |

## Managed acceptance gate

Keep this record at `managed_validation_pending` until the owner-attributed
batched Windows Release lane proves current-source compilation, accessibility
root output parity, allocation behavior, and the declared Runtime accessibility
product p50/p95/p99 gates.
