---
title: Runtime274 Borrowed Toast Timer Arming
category: zircon_runtime
report_id: Runtime274-borrowed-toast-timer-arming-2026-09-09
date: 2026-09-09
session_id: root-runtime274-borrowed-toast-timer-arming-20260909
source_plan:
  - docs/plans/optimize/zircon_runtime/274/2026-08-28-owned-toast-timer-event.md
  - docs/plans/optimize/zircon_runtime/75/2026-08-27-borrowed-toast-queue-scan.md
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime274 Borrowed Toast Timer Arming

## Scope

Toast timer admission parsed queue values and surface state into owned strings before the timer
state could decide whether ownership was needed. This slice keeps the existing timer deadlines,
queue aliases, Enum/String handling, stale-event behavior, and public owned accessor semantics.
It only changes the internal hot path used by `UiInputManager` and default toast expiry checks.

## Implementation

- `toast_timer_from_queue_value` and its Map/String helpers now return borrowed `&str` values.
  Nested arrays and all existing duration aliases remain recursive and deterministic.
- `UiSurface::toast_timer_for_component_node_ref` and borrowed metadata/state helpers avoid an ID
  clone while a default expiry action only compares the current ID.
- `UiInputTimerState::arm_toast_expiration_ref` updates an existing deadline in place. The retained
  ID is reused when it is unchanged and is owned only when a new ID must be stored. The original
  generic `arm_toast_expiration` API remains unchanged.

## TDD and local evidence

- RED: the queue pointer-identity test and timer same-ID reuse test were added before their source
  changes; the pre-change source still contained the owned parser and had no borrowed arm entry.
- Static GREEN: exact Rustfmt parse/write, source contracts, and scoped `git diff --check` pass for
  the five touched Rust owners. The focused tests cover String and Map queue storage, retained
  component-state storage, same-ID deadline refresh, and changed-ID replacement.
- The independent release model
  `.codex/state/session-coordinator/runtime274-borrowed-toast-timer-arming-model.rs` compiled
  with `rustc 1.94.1 -O` and preserved its checksum across legacy and borrowed paths.

## Allocation evidence

The model uses 4,096 queue updates with 25% zero-duration entries, 4,096 stale comparisons, and
4-entry runs of repeated IDs. It measures ownership work in isolation; it is not an end-to-end UI
latency claim.

| Workload | Legacy allocations / bytes | Borrowed allocations / bytes | Reduction |
|---|---:|---:|---:|
| Mixed queue arm | 4,096 / 40,960 | 3,072 / 30,720 | 25% |
| Stale ID comparison | 4,096 / 40,960 | 0 / 0 | 100% |
| Repeated-ID rearm | 4,096 / 32,768 | 1,024 / 8,192 | 75% |

The model emitted p50/p95/p99 samples, but the short isolated arm timings were noisy and are not
used as a product qualification threshold. Managed release p50/p95/p99 and Cargo behavior remain
required before promotion.

## Validation boundary

The nine-module Runtime Python contract batch passed `36/36`, and the same files passed
`py_compile`. No workspace Cargo or rustc validation was started. External managed admission is
still blocked because `E:/Git/zr_vm` is at HEAD
`d717af6c8fefb4f0e1a5a69423904327296f1254` with 75 dirty status entries. This record therefore
stays `managed_validation_pending`.

## Remaining scope

Runtime274 still owns full UI input dispatch and product timer qualification. This record does not
claim managed Cargo, visual/product capture, or final release percentile acceptance.
