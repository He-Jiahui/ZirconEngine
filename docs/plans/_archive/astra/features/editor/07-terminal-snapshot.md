---
status: in_progress
plan_sources:
  - docs/plans/optimize/zircon_editor/269-editor-build-export-preset-pipeline-cook-pack-platform-bundle-publishing-resume-determinism-current-working-tree-review.md
  - docs/plans/astra/performance/01-bounded-hotpaths.md
---

# Terminal snapshot retention

## Current source and repair

The successful wizard runner cloned all recorded stages and diagnostics into a
temporary pipeline execution and assigned them back to the same job snapshot.
Every stage has already recorded its latest progress and diagnostics. Completion
now computes terminal state directly from those owned records and clears live
output. The external pipeline-result method still moves its input into the job
and shares the same terminal transition.

This stays inside the existing job-state owner. Full terminal event snapshots
remain owned independently by consumers; their required copy is preserved.
Unreal's pending-message move and Fyrox's incremental output ownership evidence
are recorded in the adjacent M19 output projection plan.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M21 | Complete recorded wizard stages without cloning them into an intermediate result | implemented_pending_validation | Includes empty/1/1k/10k histories and success/failure/cancellation precedence; combined batch pending |

Tests compare the previous completion flow and retain stage/diagnostic/progress
buffer identities. The 101-sample release comparison alternates both paths after
eight warmups, completing 16 prebuilt jobs per sample. Source history is capped at
512 retained lines per stage. The before path consumes prepared progress as the
original runner did; the after path drops that same local progress. Input setup
and final job destruction stay outside timing. Report p50/p95/p99 with <=5% small
case regression and >=20% large-history p95 improvement; results remain pending.
