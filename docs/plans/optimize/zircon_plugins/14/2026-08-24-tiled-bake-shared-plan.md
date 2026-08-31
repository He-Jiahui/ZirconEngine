---
title: Plugins14 Tiled Bake Shared Plan Optimization
category: zircon_plugins
report_id: Plugins14-tiled-bake-shared-plan-2026-08-24
date: 2026-08-24
session_id: root-runtime-interface03-activate-link-failure-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Plugins14 Tiled Bake Shared Plan Optimization

## Scope

This slice advances NNAV-P1-027 for asynchronous tiled navigation bake dispatch. It only removes
redundant per-tile plan reference-count work; it does not claim the wider bounded scheduler,
priority, cancellation, persistent tile artifact, or per-world generation milestones are complete.

## Implementation

`PendingTiledBakeState` and tile workers now share one `Arc<RecastTiledBakePlan>`. A worker clones
that outer Arc once instead of cloning a plan whose mesh, flattened vertices, triangle areas, and
tile list are each stored behind separate Arcs.

Workers release their plan reference before publishing completion. A `dispatch_complete` gate keeps
the task unharvestable until every worker has been submitted and the dispatcher has also released
its plan reference. Therefore a ready task has one remaining plan owner and harvest recovers the
original plan with `Arc::try_unwrap` rather than cloning it for publication.

## Performance Evidence

| Evidence | Before | After / target | Change |
| --- | ---: | ---: | ---: |
| Arc increment/decrement pairs per dispatched tile | 4 | 1 | 75% fewer reference-count pairs |
| Completed-bake plan clones during harvest | 0 | 0 | zero-copy publication retained |
| Release benchmark P95 | pending coordinator | <= 80% of legacy | acceptance gate |

The ignored release test uses 21 alternating sample pairs and 200,000 plan clones per sample. It
prints `PERF_RESULT plugins14_tiled_bake_shared_plan` with raw samples and nearest-rank P50/P95.
No dynamic timing is accepted until the coordinator returns terminal evidence.

## Validation

- Source contract: 5/5 passed after a confirmed 3/3 initial red state.
- Exact `rustfmt --check`, Python bytecode compilation, and scoped `git diff --check`: passed.
- Existing tiled-bake behavior tests and ignored release performance evidence: pending the managed
  coordinator batch.
- No local Cargo lane was launched and no Cargo process was terminated.

### Current-source convergence receipt

- Ownership transfer preview request: `05a25f6395b24bcba5954a3a760e1433`.
- Ownership transfer apply request: `0349eac120824ec1b76eeeeacee349e5`.
- Applied fingerprint: `2e73b6aaea1d3d249d3c159cb4e803735d3995c218ed4b1ef467962cc75475e2`.
- Current session: `root-runtime-interface03-activate-link-failure-20260831`.
- Static/model ticket: `0b7e496729eb4581894c2ff6bbba09d0` (queued, 11 Python tests).
- Release performance ticket: `c402a39dec7c4e4da725980397e9d442` (queued; exact ignored Rust benchmark).
- Deterministic model: `tools/plugins14_tiled_plan_pressure.py`, source manifest `1CCD26AE31967E878B4363221F3632AF3C7EE4D7DB017A5E6EFE764D42911A64`.
- Current source hashes: `task_pool.rs` `D60A80C4FE8AFEF1BEF044D307187BF6BB3CB81E375691D2B26C73507B8180B0`; deterministic model `7E18830264D8E366274BC313FBEFEAC41A93F14A82F88C15A4A0755BD9ED482E`.

The current-source model is structural evidence, not wall-clock timing. Across 200,000 plan clones per sample it changes Arc increment/decrement pairs `800,000 -> 200,000` and modeled atomic reference-count operations `1,600,000 -> 400,000`, both `-75%`, while retaining 200,000 payload observations and zero completed-plan copies. The queued 21-pair alternating release benchmark remains authoritative for P50/P95 and must satisfy candidate P95 `<= 80%` of legacy before integration or WeCom publication.

## Remaining Parent-plan Work

NNAV-P1-027 still requires bounded admission, priority/deadline/cancel, generation-aware early abort,
and immutable geometry snapshots. NNAV-P1-024 through NNAV-P1-048 remain governed by the parent plan
and are not hidden by this allocation slice.
