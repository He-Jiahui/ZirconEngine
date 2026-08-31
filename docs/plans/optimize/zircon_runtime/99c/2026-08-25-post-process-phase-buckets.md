---
title: Runtime102 Post-process Phase Buckets
category: zircon_runtime
report_id: Runtime102-post-process-phase-buckets-2026-08-25
date: 2026-08-25
session_id: root-runtime102-post-process-phase-buckets-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime102 Post-process Phase Buckets

## Scope

This batch optimizes two graph-compilation foundations retained by the Runtime102/99c post-process
plan: deterministic pass ordering and resolved view-family phase validation. It does not implement
post-process executors, GPU work submission, history ownership, output transfer, capture fixtures,
or the parent plan's cross-platform product qualification gates.

## Implementation

`ordered_node_indices` now admits nodes into eight canonical phase buckets and runs the stable Kahn
sort only over explicit same-phase dependencies. Canonical ascending bucket traversal supplies the
lower-phase-before-higher-phase invariant without synthesizing and storing every cross-phase edge.
Explicit dependencies on a later phase still report `CycleDetected`, and dependency resolution is
completed first so a later `MissingDependency` retains its prior error precedence.

`validate_view_family_phases` now builds one enabled-phase mask and records observed phases during
the existing node availability pass. Required temporal/spatial phases are checked against that
mask instead of rescanning all post-process nodes.

## Performance Evidence

| Evidence | Before | After / target | Result |
| --- | ---: | ---: | ---: |
| 1,024 independent pass nodes | 1,048,576 phase comparisons plus synthesized cross-phase edges | 1,024 phase admissions plus empty same-phase edge sets | quadratic phase-edge construction removed |
| Phase ordering release P95 | unbounded | <= 10% of legacy and <= 5 ms | pending terminal evidence |
| 16,385 view-phase nodes | enabled-phase slice search per node plus a second required-phase node scan | one enabled mask plus one node pass | repeated node scan removed |
| View-phase validation release P95 | unbounded | <= 75% of legacy and <= 5 ms | pending terminal evidence |

The ignored Windows release tests alternate 15 legacy/optimized sample pairs and print raw sample
vectors plus nearest-rank P50/P95 under
`RUNTIME102_POST_PROCESS_PHASE_BUCKET_BENCH_V1` and
`RUNTIME102_POST_PROCESS_VIEW_PHASE_SINGLE_PASS_BENCH_V1`. Exact latency and reduction percentages
are accepted only from the coordinator's terminal output.

## Validation

- RED source contracts recorded the nested all-node phase comparison and the required-phase node
  rescan in production.
- Correctness coverage compares optimized output with the retained legacy reference for independent
  phases and same-phase dependencies.
- Error coverage fixes missing-dependency precedence and later-phase dependency cycle behavior.
- Static GREEN confirms fixed phase buckets, same-phase-only stored edges, and single-pass phase
  masks.
- Scoped `rustfmt` and `git diff --check` are prepared locally.
- Focused release correctness and performance tests are prepared for one managed Runtime batch.
- Terminal marker data, integration commit, and coordinator-owned WeCom delivery remain pending.

## Documentation Decision

Public graph DTOs, validation error variants, pass ordering, and view-family contracts are
unchanged. This numbered optimization record is sufficient for the internal planner change.

## Remaining Parent-plan Work

Runtime102/99c still requires real execution ownership for the retained passes, shader/backend
integration, resource and temporal-history lifetime enforcement, output-transfer validation,
readback/capture evidence, and full product-scale qualification across supported platforms.
