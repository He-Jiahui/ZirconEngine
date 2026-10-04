---
title: Runtime200 focus pointer-drag owner in-place retention
category: zircon_runtime
report_id: Runtime822-focus-pointer-drag-retain-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime822 · focus pointer-drag owner in-place retention

## Scope

`UiSurface::clear_invalid_transient_input_owners` previously copied every
invalid pointer-drag owner into a temporary `Vec`, then performed a second
map lookup/removal for each owner. Tree reconciliation can invalidate many
owners at once, so this created avoidable allocation and traversal work on the
transient-input cleanup path.

## Implementation

- Move `input.pointer_drags` out with `std::mem::take`, remove invalid owners
  with `BTreeMap::retain`, and move the same map back.
- Preserve input-owner validation, the surviving key/value pairs, key order,
  and the existing empty-map behavior; only the temporary owner list and
  repeated removals are removed.
- Add a lower Rust semantic regression and the ignored
  `RUNTIME822_FOCUS_POINTER_DRAG_RETAIN_BENCH_V1` marker for the managed
  Release lane.

## TDD and deterministic model

The Python source/model contract was intentionally RED against the temporary
owner-vector path and GREEN after the in-place map retention was added. For
4,096 pointer-drag entries and 1,000 reconciliations, the legacy shape creates
one temporary vector per reconciliation and performs one map removal lookup
for every invalid entry. The optimized shape creates no temporary owner vector
and performs one `BTreeMap::retain` traversal per reconciliation. The model is
allocation/complexity evidence only; it is not allocator, CPU, RSS, or product
p50/p95/p99 evidence.

## Local evidence

- Focused source/model contract:
  `tools/tests/test_runtime_focus_pointer_drag_retain_performance_contract.py`
  (`4/4`).
- Lower Rust semantic regression and ignored Release marker are wired in
  `focus.rs`.
- Exact-file Rustfmt and Python compilation pass. The current one-process
  non-tooling Runtime/Editor batch loads `621` modules and passes `2218/2218`
  tests with zero failures, errors, or skips; a later shared batch including
  Runtime824, Editor827, and Editor828 loads `624` modules and passes
  `2227/2227` with zero failures, errors, or skips. Managed Cargo, Windows
  Release, and product input percentile evidence remain pending.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/surface/focus.rs` | `291C19BC5AA2E7381C6DF15BD75350BC3B7E8163F5F8B313CFD1E63276DAAEC8` |
| `tools/tests/test_runtime_focus_pointer_drag_retain_performance_contract.py` | `3EC61B223C98C3898B42134F3183724A548D71A512C339CAFED91F4B8BF558D4` |

## Acceptance boundary

Keep this record `implementation_complete` /
`managed_validation_pending` until the owner-attributed Windows Release batch
compiles the current Runtime/Editor tree, executes the lower regression and
ignored marker, and supplies input allocation plus product p50/p95/p99
evidence. Tooling production remains deferred for the later Rust migration.
