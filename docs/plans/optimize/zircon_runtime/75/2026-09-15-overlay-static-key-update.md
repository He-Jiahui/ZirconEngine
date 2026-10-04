---
title: Runtime75 Overlay Static Key Update
category: zircon_runtime
report_id: Runtime75-overlay-static-key-update-2026-09-15
date: 2026-09-15
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Overlay Static Key Update

## Scope

Popup and dialog state transitions publish a fixed set of keys (`popup_open`, `open`, anchor
coordinates, `confirmed`, and `dialog_action_id`). The previous reducer path built a new `String`
key for every update and delegated to the generic setter, even when the key was already present in
the retained component state.

## Implementation

- Added `set_overlay_value`, which clears reference provenance and mutates an existing map value in
  place, allocating an owned key only for a first insertion.
- Routed popup-anchor, dialog-commit, and open-alias publication through that helper.
- Preserved alias precedence, descriptor/state presence checks, popup flags, and dialog action
  semantics.
- Added lower key-identity/reference-source regressions and ignored marker
  `RUNTIME783_OVERLAY_STATIC_KEY_UPDATE_BENCH_V1`.

## Deterministic work model

For an already materialized overlay state, each fixed-key update changes from one temporary key
allocation to zero. Missing keys still use the same owned insertion boundary. This is
allocation-shape evidence only, not a claim about allocator, CPU/RSS, or product popup latency
percentiles.

## Validation

- TDD source contract:
  `tools/tests/test_runtime_overlay_static_key_update_performance_contract.py` was RED before
  implementation and is GREEN at `3/3`.
- The current-source Runtime/Editor contract loader passes `1948/1948` across `543` modules, and
  the focused Runtime/Editor hot-path set passes `142/142` in one process. These are local
  source/model receipts only.
- `rustfmt --edition 2021 --check` passes for the production overlay reducer and lower regression.
- Managed Cargo, Release allocation, and product p50/p95/p99 evidence remain pending in the
  owner-attributed Runtime/Editor batch.

## Remaining acceptance

The owner-attributed Windows Runtime/Editor lane must compile the current source, execute the lower
regression and ignored marker, and report the plan-specific allocation and latency gates.
