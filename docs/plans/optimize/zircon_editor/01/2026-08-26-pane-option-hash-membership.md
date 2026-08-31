---
title: Editor01 Pane Option Hash Membership
category: zircon_editor
report_id: Editor01-pane-option-hash-membership-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Pane Option Hash Membership

## Scope

This slice removes logarithmic state membership from retained pane option projection. Selected,
disabled, special, focused, hovered, pressed, and loading sets are private lookup indexes; option
publication continues to follow the input option vector.

## Change

- Replace pane-option `BTreeSet<String>` lookup sets with `HashSet<String>`.
- Apply the same hash type to normalized selected values and all state-attribute projections.
- Preserve option parsing, labels/flags, selected aliases, focused index, hover ID, query matching,
  and input publication order.

## Deterministic Performance Evidence

| Representative 8,192 option IDs / 65,536 lookups | Before | After |
|---|---:|---:|
| Membership class | O(log n) | average O(1) |
| Published option order | input order | input order |
| State channels using the index | 7 | 7 |

The ignored release gate runs 17 alternating samples and emits
`EDITOR01_PANE_OPTION_HASH_MEMBERSHIP_BENCH_V1`. Acceptance requires hash membership P95 to be at
most 60% of ordered membership P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826z_editor01_pane_option_hash_sets_preserve_state_and_input_order`
  exercises selected/disabled state and publication order through product projection.
- `optimization_batch_20260826z_editor01_pane_option_projection_uses_hash_membership` requires the
  private production hash boundaries and rejects ordered membership.
- `optimization_batch_20260826z_editor01_pane_option_hash_membership_performance_evidence` checks
  lookup equivalence, reports both P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Editor01 still needs generation-aware pane projection invalidation, resize-only conversion bypass,
virtualized large option surfaces, retained allocation telemetry, and full input-to-paint latency
qualification. This slice only improves per-option state membership.
