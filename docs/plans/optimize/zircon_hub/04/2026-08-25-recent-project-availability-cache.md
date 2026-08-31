---
title: Hub04 recent-project availability cache
plan: docs/plans/optimize/zircon_hub/04-command-action-message-delivery-task-history-view-model-localization-product-integration-review.md
session: root-hub04-two-task-performance-batch-r3-20260831
date: 2026-08-25
status: implementation_complete_managed_validation_pending
---

# Hub04 recent-project availability cache

## Scope

This slice addresses `ZHUB-CTL-P1-44`: repeated Hub read-model projection must not
perform synchronous filesystem probes for every recent project. The change remains
inside the existing Hub snapshot/runtime-session/view-model ownership boundary.

## Implementation

- `ProjectAvailabilitySnapshot` owns the path-to-availability projection.
- Session startup captures each recent path once.
- Ordinary `view_model()` calls synchronize only additions/removals. An unchanged
  recent-project set performs an O(n) membership pass with no filesystem probe or
  temporary collection. Changed sets rebuild the cache once and probe only additions.
- Window-focus refresh rechecks all current paths and emits a new view model when
  availability changes even if the shared recent-project list is unchanged.
- Existing/missing filtering and selected-project detail projection consume the same
  cached snapshot. A selected path outside the recent-project list is admitted to that
  snapshot and probed once, so an existing stale selection remains available without
  reintroducing a filesystem probe on every view-model projection.

## TDD evidence

The original source contract was introduced before production changes and failed `4/4`.
The r3 continuation added contracts for the complete repeated-projection path before
changing Rust; they failed because synchronization was O(n squared) and the benchmark
did not include synchronization. After implementation and the selected-path regression repair:

- `python -m unittest tools.tests.test_hub04_recent_project_availability_performance_contract -v`
  passes `5/5`.
- The combined Hub04 source-contract batch passes `9/9`.
- Scoped `rustfmt +1.94.1 --edition 2021 --check` passes.
- Scoped `git diff --check` passes; Git reports only the repository's expected
  LF-to-CRLF checkout warning.

The Rust behavior suite adds coverage that unchanged project sets do not probe again,
new paths are probed exactly once, a selected path outside recents is cached without
repeat probes, and cached availability preserves filter results.

## Performance evidence

Local Windows-native release preflight used 1,000 missing project paths, 21 alternating
sample pairs, nearest-rank percentiles, and excluded the one-time cache capture from the
repeated-projection measurement.

| Metric | Legacy per-projection probe | Cached projection | Reduction |
|---|---:|---:|---:|
| P50 | 372,833,500 ns | 592,500 ns | 99.841% |
| P95 | 582,366,100 ns | 1,010,300 ns | 99.827% |
| Filesystem probes per projection | 1,000 | 0 | 100% |

The managed Rust release gate remains stricter and is not replaced by this preflight:
10,000 missing paths, 21 alternating sample pairs, exact output equality, the O(n)
synchronization step included in every optimized sample, zero optimized filesystem
probes per repeated projection, and optimized P95 at least 40% below legacy.

## Validation and integration

- Direct Cargo was not used. The exact combined Hub04 Rust batch will be submitted to the
  Windows coordinator as:
  `cargo +1.94.1 test --manifest-path zircon_hub/Cargo.toml --lib --test hub04_message_id_lookup_performance --locked --release --jobs 1 -- hub04_ --include-ignored --nocapture --test-threads=1`.
- Coordinator validation, independent review, integration commit, and automatic WeCom
  delivery remain pending. The final WeCom message must include the managed P50/P95 and
  filesystem-probe reductions.
- The first r3 validation snapshot predates the selected-path regression repair and is
  superseded for terminal acceptance. A replacement snapshot must include the new
  `hub04_project_availability_caches_selected_path_outside_recents` behavior contract.
- The same Hub04 Session already owns commit `0e2e980b9` for the earlier page-projection
  slice. Its existing async validation state remains intact; this record does not claim
  that prior coordinator work is closed.
