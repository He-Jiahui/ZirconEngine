---
title: Runtime runtime-tree pseudo-state collector capacity
category: zircon_runtime
report_id: Runtime841-runtime-pseudo-state-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime841 · runtime-tree pseudo-state collector capacity

## Scope

`collect_runtime_pseudo_states` rebuilds the runtime-tree selector state list
when component or node flags change. The old collector started at zero
capacity even though the authored attribute count, enabled flags, alias fanout,
and resolved painter aliases provide a bounded upper estimate.

The collector now reserves a saturating bound before collecting attributes and
runtime flags. The bound counts the maximum alias fanout of each currently
enabled flag, reserves two slots for the resolved painter aliases, and keeps
empty/clean nodes at the small painter-only bound. Sorting, deduplication,
retained-state filtering, and painter-family semantics are unchanged.
The helper-local fanout values are named private constants; they are not a
cross-crate policy contract.

## TDD and deterministic model

- The Python source/model contract was intentionally RED against the original
  `Vec::new()` collector and GREEN at `4/4` after the helper, lower-test
  wiring, and Release marker were added.
- The lower Rust order/capacity regression and ignored
  `RUNTIME841_RUNTIME_PSEUDO_STATE_CAPACITY_BENCH_V1` marker are wired in
  `runtime_state/capacity_tests.rs`.
- For 4,096 non-retained authored names plus the all-enabled component/node
  alias allowance, the structural model reserves `4,122` slots. The old
  zero-capacity collector models `12` geometric growth events; the bounded
  collector models `12→0`. This is allocation-shape evidence only, not
  allocator, CPU, RSS, or product p50/p95/p99 evidence.

## Local evidence

- Runtime841 source/model contract: `4/4`.
- Runtime810 regression plus Runtime841 contract: `7/7`; the combined
  Runtime841/Runtime810/Runtime840/Editor840/Editor09/Runtime19 batch passes
  `30/30`, and the current non-tooling performance-contract loader passes
  `2340/2340` across `640` modules in `7.026s`.
- Exact-file Rustfmt and Python compilation pass.
- Source SHA-256:
  `4EC5919AEBB62FCBABE5168CFB47F94769D58D0214ECBC504F7E20C7D19D3712`.
- Lower-test SHA-256:
  `2CF1504E6360BA6F14AB361D6F9FD41DEA0C97119338BC4F761E3BAE529AF9EB`.
- Contract SHA-256:
  `D9FCBD5A9DAC4ED092C395A1711D63752E97166026B96DA25BA67C5375FD88D4`.

Managed Windows Cargo/Release execution, lower Rust execution, allocator
measurements, and product style p50/p95/p99 evidence remain pending under the
shared external-worktree admission gate. Tooling production remains deferred.

## Managed acceptance gate

Keep this record at `managed_validation_pending` until the owner-attributed
batched Windows Release lane proves runtime-tree state parity, allocation
behavior, and the declared Runtime73 style product p50/p95/p99 gates.
