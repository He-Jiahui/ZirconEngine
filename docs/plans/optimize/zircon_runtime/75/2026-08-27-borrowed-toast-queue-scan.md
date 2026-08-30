# Runtime75 Borrowed Toast Queue Scan Optimization Record

- Date: 2026-08-27
- Owner: `root-runtime75-borrowed-toast-queue-scan-20260827`
- Source plan: `docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md`, RUW-P1-041 / RUW-P1-047
- Status: implementation and isolated release-model evidence complete; managed Cargo validation pending

## Problem

Every Toast state synchronization recursively converted the full queue into an
owned `Vec<ToastEntry>`. String fields and each raw `UiValue` were cloned even
though synchronization only needed the queue length and one selected entry.
Nested arrays also created intermediate vectors through recursive
`flat_map(...).collect()` calls. Expiry repeated the same parsed-field cloning
before retaining only the raw values that survived removal.

## Change

- `visit_toast_entries` recursively visits valid queue leaves without an
  intermediate collection.
- `BorrowedToastEntry` keeps parsed string slices and a borrowed raw projection.
- Synchronization counts all entries but materializes only the selected display
  entry after the borrowed scan ends.
- Expiry clones only retained raw values and never clones discarded parsed
  fields.
- `BorrowedToastRaw` preserves existing transport behavior: String and Enum
  leaves normalize to `UiValue::String`, while Map leaves remain maps.
- Focused Rust tests cover nested-array flattening, current-ID selection, and
  Enum-to-String expiry normalization.

## Performance Evidence

Isolated `rustc -O` model:
`.codex/state/session-coordinator/runtime75-borrowed-toast-queue-scan-model.rs`.
The workload contains 512 string-backed Toast entries in 64 nested groups.

| Sync scan metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 3,207 | 3 | 99.91% |
| Allocated bytes | 370,368 | 115 | 99.97% |
| P50 | 620,200 ns | 86,900 ns | 85.99% |
| P95 | 755,000 ns | 99,100 ns | 86.87% |

| Expiry filter metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 3,207 | 519 | 83.82% |
| Allocated bytes | 370,368 | 100,092 | 72.97% |
| P50 | 623,800 ns | 144,100 ns | 76.90% |
| P95 | 4,774,000 ns | 225,500 ns | 95.28% |

The legacy and optimized model checksums are identical (`1,138`). These values
isolate queue scan/filter work and do not claim end-to-end UI event latency.

## Validation

- `python -m unittest tools.tests.test_runtime75_borrowed_toast_queue_scan_performance_contract`: 3/3 passed.
- Exact-file `rustfmt`: passed.
- Standalone release model compiled with `rustc 1.94.1 -O`: passed.
- Managed Cargo regression remains pending in the next combined coordinator
  batch. Current coordinator materialization is globally blocked by foreign
  unmanaged Tooling15 build roots; no prior ticket is replayed.

## Remaining Scope

Toast/Notification typed-model convergence, timer ownership, pause/dismiss
transactions, accessibility announcement, and live-surface integration remain
RUW-P1-041 / RUW-GATE-041 work. This change only removes queue projection
allocation pressure from the existing reducer behavior.
