---
title: Runtime04 Migration Report Format Capacity
category: zircon_runtime
report_id: Runtime04-migration-report-format-capacity-2026-08-26
date: 2026-08-26
session_id: optimize-runtime04-direct-reference-batch-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime04 Migration Report Format Capacity

## Scope

`AssetMigrationReport::format_text` emits a bounded header plus one line per changed file and
issue. The old `String::new()` left the output buffer to grow geometrically as reports scaled.

## Implementation

The formatter now estimates capacity from the fixed header and the known change/issue counts,
using saturating arithmetic before one `String::with_capacity` allocation. Formatting order,
field values, path display normalization, and the public text contract remain unchanged. The
estimate is deliberately a lower-bound heuristic; long paths/messages may still grow normally.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Initial output allocation | `String::new()` | one count-based reservation |
| Change/issue traversal | unchanged | unchanged |
| Report text | legacy output | byte-identical |
| Release p95 | dynamic evidence pending | optimized p95 <= 2x legacy guard; values from coordinator |

The ignored release test prints `RUNTIME04_MIGRATION_REPORT_FORMAT_CAPACITY_BENCH_V1` with
alternating legacy/optimized p95 samples, report sizes, reserved rows, and the iteration count.
It compares output against a test-only legacy formatter, so the guard does not rely on a synthetic
allocation claim.

## Validation

- The source contract was RED before the reservation and GREEN 3/3 afterward.
- Scoped Rustfmt and `git diff --check` passed for `report.rs`.
- Functional coverage checks byte-for-byte equivalence against the previous formatter for mixed
  change/issue reports.
- Managed release command:
  `cargo +1.94.1 test -p zircon_runtime --locked --lib --release runtime04_migration_report_ -- --include-ignored --nocapture --test-threads=1`
- Commit integration, terminal p95 values, record finalization, and WeCom delivery remain
  coordinator-owned.

## Remaining Parent-plan Work

Runtime04 still requires the parent migration-index scale matrix and all managed validation gates;
this output-allocation slice does not close those broader requirements.
