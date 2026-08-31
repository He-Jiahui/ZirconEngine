---
title: Runtime08C Lazy Missing Track Path Materialization
category: zircon_runtime
report_id: Runtime08C-lazy-missing-track-path-2026-08-26
date: 2026-08-26
session_id: root-runtime08c-lazy-missing-track-path-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime08C Lazy Missing Track Path Materialization

## Scope

This slice removes diagnostic-path construction from the successful animation-sequence track
compile path. It does not change target resolution, writer compilation, track ordering,
binding/track indices, missing-target diagnostics, missing-writer diagnostics, catalog generation,
compiled writer ownership, apply behavior, or public APIs.

The adjacent animation manager and sequence test files contain unrelated active work and are
deliberately not edited. Existing compiled-sequence tests remain read-only behavior oracles.

## Change

- `compile_sequence_for_world` now asks the world for a compiled property writer before creating an
  `AnimationTrackPath` diagnostic.
- A resolved writer publishes the same `CompiledAnimationSequenceTrack` without cloning entity or
  property paths and without formatting a diagnostic string that would be immediately dropped.
- A missing writer constructs and stores the same `AnimationTrackPath` as before, then continues.
- The already-missing-entity branch retains its existing per-track diagnostics unchanged.
- A Python source contract prevents eager path construction from returning and pins one successful
  writer test plus one missing-target test.

## Deterministic Performance Evidence

The independent release model mirrors the real owned shape of `EntityPath`,
`ComponentPropertyPath`, and `AnimationTrackPath`. It compiles 1,024 successful tracks per
operation, runs 256 operations per sample, and uses 21 paired samples with alternating order.

| Evidence | Eager diagnostic path | Lazy missing-only path | Result |
| --- | ---: | ---: | ---: |
| Allocations per 1,024 successful tracks | 13,312 | 0 | 100% removed |
| Missing-path count | 1,024 | 1,024 | identical |
| Missing-path length checksum | 55,210 | 55,210 | identical |
| Run 1 P50 | 409.293 ms | 0.202 ms | 99.951% faster |
| Run 1 P95 | 589.118 ms | 0.351 ms | 99.940% faster |
| Run 2 P50 | 420.583 ms | 0.184 ms | 99.956% faster |
| Run 2 P95 | 562.554 ms | 0.220 ms | 99.961% faster |
| Run 3 P50 | 480.100 ms | 0.194 ms | 99.960% faster |
| Run 3 P95 | 670.588 ms | 0.580 ms | 99.914% faster |

The timing row intentionally isolates the deleted diagnostic construction; it is not an
end-to-end claim for world writer compilation. The managed gate requires exact allocation counts
of 13,312 and 0, identical missing-path count/checksum, and at least 99% reduction in both focused
P50 and P95 construction time.

## Acceptance

- TDD RED observed two eager-construction contract failures while the existing Rust behavior
  oracle check passed.
- `tools.tests.test_runtime08c_lazy_missing_track_path_performance_contract` passes 3/3 locally.
- Exact production/model `rustfmt --check`, Python compilation, PowerShell parsing, three paired
  model runs, and scoped diff checks pass locally.
- All compiled-sequence Rust tests, source contracts, formatting, the performance model, and scoped
  diff checks are submitted together in one coordinator validation ticket with one Cargo command.
- Commit integration and automatic WeCom performance notification remain gated on managed
  validation and the repository's independent-review policy.

## Remaining Parent-plan Work

Runtime08C still owns prepared animation artifacts, dense pose storage, residency, candidate
indexing, worker affinity, graph/state-machine execution, event indexing, sequence mutation
buffering, IK phases, GPU skinning handoff, Editor authoring, and product-scale qualification.
