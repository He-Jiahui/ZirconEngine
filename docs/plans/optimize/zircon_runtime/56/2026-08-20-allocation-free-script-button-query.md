# Runtime56 Allocation-Free Script Button Query

- Date: 2026-08-20
- Session: `optimize-runtime56-direct-button-query-r1-01a00797-20260820`
- Findings: `INP-P1-005`, `INP-P1-046` (snapshot-clone hot-path slice only)
- Performance marker: `PERF-MVP-559`
- Status: implementation complete; managed batch validation and release measurements pending

## Scope

The gameplay `key_pressed` host function previously resolved the input manager and then cloned an
`InputSnapshot` for every individual key query. `DefaultInputManager::snapshot` materializes every
pressed button into a new `Vec`, so four WASD checks in one script tick repeated the same allocation
and clone work.

`InputManager` now exposes a single-button observation method with a snapshot-based compatibility
default. `DefaultInputManager` overrides it with a direct lookup in the existing `ButtonInputState`
under the same state lock, and the script host compiles its string argument to one typed
`InputButton` before calling that method. Full frame-context manager resolution and typed action
handles remain open under `INP-P1-005`; this candidate does not claim to close that broader item.

## Deterministic Work Reduction

The release workload retains 1,024 pressed key codes and performs 2,048 missing-key queries per
sample. The legacy path creates 2,048 snapshot vectors and clones 2,097,152 button values per
sample. The direct path creates zero snapshot vectors and clones zero retained buttons; it performs
the existing ordered-set lookup while holding the same manager state lock.

These allocation and clone counts are deterministic and are not timing claims. Release latency
remains pending until the managed validator records actual samples.

## Acceptance Contract

- Behavior tests require present and missing direct queries to match the previous snapshot result.
- Script key-code and named-key parsing must continue to produce the same typed buttons.
- The script hot-path source contract rejects reintroduction of `input.snapshot()`.
- The ignored release benchmark runs 21 legacy/direct sample pairs and alternates which path runs
  first.
- Each sample performs 2,048 queries against 1,024 retained buttons.
- P50 and P95 use nearest-rank selection.
- Direct-query P95 must be at most 25% of legacy snapshot-query P95.
- The managed multi-task validator must parse both raw sample vectors and independently recompute
  percentile, deterministic-work, and threshold checks before this record can be marked accepted.

## Validation

Scoped formatting and diff checks are required before the candidate snapshot. Cargo tests and
release performance measurements are intentionally deferred to a managed multi-task validation
batch; no passing result or measured latency is claimed in this record yet.
