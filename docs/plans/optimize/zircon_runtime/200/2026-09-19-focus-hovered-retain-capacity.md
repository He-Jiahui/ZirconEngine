---
title: Runtime200 focus hovered-path in-place retention
category: zircon_runtime
report_id: Runtime821-focus-hovered-retain-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime821 · focus hovered-path in-place retention

## Scope

`UiSurface::clear_invalid_transient_input_owners` runs after a tree change and
rebuilds the retained hovered path with `filter(...).collect()`. Even when all
hovered nodes remain valid, that shape allocates a replacement `Vec` and drops
the previous buffer. The hovered path is already bounded by the published hit
path, so its existing allocation is a safe reusable scratch buffer.

## Implementation

- Move `focus.hovered` out with `std::mem::take`, filter it in place with
  `Vec::retain`, and move the same buffer back.
- Preserve input-owner validation, duplicate entries, source order, and the
  existing empty-path behavior; only the allocation lifetime changes.
- Add a lower Rust order/capacity regression and the ignored
  `RUNTIME821_FOCUS_HOVERED_RETAIN_BENCH_V1` marker for the managed Release
  lane.

## TDD and deterministic model

The Python source/model contract was intentionally RED against the old
replacement-vector path and GREEN after the in-place retention was added. For
4,096 retained hover entries, the legacy shape performs one replacement-vector
allocation per reconciliation while the optimized shape performs zero new
allocations and keeps the existing capacity. The model is allocation-shape
evidence only; it is not allocator, CPU, RSS, or product p50/p95/p99 evidence.

## Local evidence

- Focused source/model contract:
  `tools/tests/test_runtime_focus_hovered_retain_performance_contract.py`
  (`4/4`).
- Lower Rust regression and ignored Release marker are wired in `focus.rs`.
- Exact-file Rustfmt and Python compilation pass. A one-process Runtime/Editor
  batch loads `393` modules and passes `1416/1416` tests; the broader current
  non-tooling batch loads `619` modules and passes `2210/2210` tests, all with
  zero failures, errors, or skips. These are source/model receipts only;
  managed Cargo, Windows Release, and product input percentile evidence remain
  pending.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/surface/focus.rs` | `F656DA41FCFB123C9A71BF22A9A50483ACC27CD3A7B82286161D0D2BD5A753C6` |
| `tools/tests/test_runtime_focus_hovered_retain_performance_contract.py` | `D45FE3CAF61D3BC7848B9990C5ADD10C67292823C6B06755F26DF515564497E9` |

## Acceptance boundary

Keep this record `implementation_complete` /
`managed_validation_pending` until the owner-attributed Windows Release batch
compiles the current Runtime/Editor tree, executes the lower regression and
ignored marker, and supplies input allocation plus product p50/p95/p99
evidence. Tooling production remains deferred for the later Rust migration.
