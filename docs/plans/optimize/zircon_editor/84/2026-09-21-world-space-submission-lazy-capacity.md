---
title: Editor World-Space Submission Lazy Capacity
category: zircon_editor
report_id: Editor879-world-space-submission-lazy-capacity-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor879 World-Space Submission Lazy Capacity

## Finding

Editor84 already streams each pane's world-space submissions directly into the
scene accumulator, removing temporary vectors and local sorts. Its filtered
iterator, however, has a zero lower size bound, so dense eligible node groups
can still grow the retained submission vector geometrically even though the
authored node count is a safe upper bound.

## Optimization

- Record the caller-owned destination length before visiting a node group.
- On the first enabled node that materializes a valid world-space submission,
  reserve the complete authored node-count upper bound once.
- Keep the destination untouched for empty, screen-only, or invalid-extent
  groups, preserving their zero-allocation behavior.
- Continue appending directly in node order. The standalone builder and final
  scene owner retain their existing sort contracts.

The retained vector remains caller-owned, and an existing prefix is neither
reordered nor recopied.

## TDD and deterministic evidence

The Editor879 source/model contract was observed RED at `1/5` and GREEN at
`5/5`. The lower regression verifies an all-screen group keeps an empty vector
at zero capacity, then appends 64 dense world nodes behind an existing prefix
without changing append order.

The one-time v9 receipt subsequently compiled Runtime, then exposed E0599 in
this owner because `ModelRc<TemplatePaneNodeData>` reports its authored length
through `row_count()` rather than `len()`. The contract was tightened first and
observed RED `4/5`; the production reserve now uses `nodes.row_count()` and the
contract is GREEN `5/5` again without changing the capacity bound.

For 4,096 retained submissions, the retired zero-lower-bound growth model
performs 11 geometric capacity changes and the lazy upper-bound model performs
zero. The ignored 101-pair Release marker
`EDITOR879_WORLD_SPACE_SUBMISSION_CAPACITY_BENCH_V1` emits alternating
p50/p95/p99 samples, locks both growth counts, and requires reserved p95 to stay
within 10% of the streaming-growth model.

## Local validation boundary

- Exact-file Rustfmt and scoped `git diff --check` pass.
- The source/model contract passes `5/5`; lower Rust execution is reserved for
  the managed batch.
- Editor879 landed after v9 and was submitted with Editor880 in asynchronous
  v10 (PID `21968`) before the v9 E0599 repair. v10 is not monitored and may
  contain the pre-repair snapshot. The repair was submitted with Editor881 in
  asynchronous v11 (PID `14240`). No per-task coordinator lane is launched.
- Local evidence does not establish Windows compilation, allocator behavior,
  or editor world-space UI product p50/p95/p99 latency.
- Tooling production remains deferred for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/host_contract/data/world_space_submission/builder/node.rs` | `9525A2CABE50F201BDC2102A27714EA453BB7CD28E13DFA4182E2824308BD45D` |
| `zircon_editor/src/ui/retained_host/host_contract/data/world_space_submission/builder/node/capacity_tests.rs` | `55DBB63F55E9EEE1D58B310B1333C57C4164A3D71FB0E8704C424BA1C8562710` |
| `tools/tests/test_editor879_world_space_submission_capacity_performance_contract.py` | `3D46A3E95927DF8EFC3CCD5B0008A7ABC51225359336B2E1B3F824A4E2BDB5D5` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
a combined current-source Windows lane compiles Editor, executes the lower
regression and ignored Release marker, and supplies allocator plus editor
world-space UI product p50/p95/p99 evidence. The deterministic growth model is
not product acceptance.
