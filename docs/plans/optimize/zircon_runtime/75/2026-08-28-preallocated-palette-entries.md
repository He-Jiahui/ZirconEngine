---
title: Runtime75 Preallocated Palette Entries
category: zircon_runtime
report_id: Runtime75-preallocated-palette-entries-2026-08-28
date: 2026-08-28
session_id: root-runtime75-borrowed-toast-queue-scan-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Preallocated Palette Entries

## Scope

This slice removes geometric Vec growth from capability-filtered component palette projection. It
contributes a focused 258-descriptor allocation and timing data point toward RUW-P1-047 and
RUW-GATE-047. It does not close component authority, descriptor-driven execution, conformance, or
the parent plan's full widget workload matrix.

## Implementation

`palette_entries_for_host` now reserves the registry length as a safe result upper bound, then
extends that Vec from the existing capability filter and palette-metadata projection. The filter,
owned entry fields, and four-key deterministic sort remain unchanged.

Two Rust regressions use the process-wide showcase registry to verify that projection retains the
registry upper-bound capacity and remains ordered by category, sort key, display name, and
component ID.

## Performance Evidence

The release model isolates result-container growth for the current combined-catalog scale of 258
descriptors and an 80% admission ratio, yielding 206 fixed-size entries. Each of 31 alternating
sample pairs performs 2,000 projections after five warmups. A counting global allocator measures
one projection, and both implementations must produce identical output and checksums. The
acceptance figures use the final rerun.

| Metric | Before | After | Change |
| --- | ---: | ---: | ---: |
| Vec allocation/reallocation calls | 7 | 1 | -85.714% |
| Vec requested bytes | 32,512 | 16,512 | -49.213% |
| Result length | 206 | 206 | unchanged |
| Result capacity | 256 | 258 | upper bound reserved |
| P50 per 2,000 projections | 5,245,100 ns | 2,412,400 ns | -54.007% |
| P95 per 2,000 projections | 11,624,200 ns | 5,904,100 ns | -49.209% |

Both implementations produced checksum `3,288,852,000`. A preceding independent run measured
P50 `5,033,200 -> 2,569,400 ns` (-48.951%) and P95 `11,514,200 -> 7,291,100 ns`
(-36.678%). This model intentionally excludes palette string/template cloning so it measures the
container policy changed by this slice; those necessary owned-field clones remain in production.

## Validation

- Source contract: 3/3 passed after a confirmed 0/3 initial state.
- Exact Rust formatting and Python contract compilation: passed.
- Scoped `git diff --check`: passed for the exact three candidate paths.
- Two Rust unit regressions were added but have not yet been executed by Cargo.
- This candidate will be grouped with allocation-free component categories in one managed
  Runtime75 validation batch; no local Cargo lane was launched.
- Commit and WeCom publication remain pending independent review and managed validation.

## Remaining Parent-plan Work

Runtime75 still requires a single component authority, descriptor admission and live behavior
convergence, schema-safe state transactions, component-specific accessibility, product-asset
migration, and the full conformance/fault/soak/scale gates. This local container preallocation does
not substitute for those gates.
