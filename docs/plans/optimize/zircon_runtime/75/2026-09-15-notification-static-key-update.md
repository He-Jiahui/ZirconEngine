---
title: Runtime75 Notification Static Key Update
category: zircon_runtime
report_id: Runtime75-notification-static-key-update-2026-09-15
date: 2026-09-15
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Notification Static Key Update

## Scope

Notification-center selection, focus, and unread-count publication repeatedly converted fixed
property names into owned `String` keys before calling the generic state setter. Those keys are
already retained in the component state during normal keyboard and selection traffic.

## Implementation

- Added `set_notification_value`, which clears reference provenance and updates an existing map value
  in place, allocating an owned key only when the property is first inserted.
- Routed selected-notification, focused-index, and unread-count publication through the helper.
- Preserved disabled-entry navigation, selection precedence, focus flags, and unread-count semantics.
- Added lower key-identity/reference-source regressions and ignored marker
  `RUNTIME784_NOTIFICATION_STATIC_KEY_UPDATE_BENCH_V1`.

## Deterministic work model

For a materialized notification state, each fixed-key publication removes one temporary key
allocation. The first insertion retains the existing ownership boundary. This is allocation-shape
evidence only, not a claim about allocator, CPU/RSS, or product notification latency percentiles.

## Validation

- TDD source contract:
  `tools/tests/test_runtime_notification_static_key_update_performance_contract.py` was RED
  before implementation and is GREEN at `3/3`.
- The current-source Runtime/Editor performance-contract loader passes `1951/1951` across `544`
  modules in `8.523s`, and the merged focused Runtime/Editor hot-path set passes `145/145` in
  `0.150s` in one process. These are local source/model receipts only.
- The adjacent batched Runtime/Editor input and compile-contract probe passes `176/176` across
  `24` modules in `8.157s`; this is source-contract evidence, not a Rust Cargo compile.
- `rustfmt --edition 2021 --check` passes for the notification reducer and lower regression.
- Managed Cargo, Release allocation, and product p50/p95/p99 evidence remain pending in the
  owner-attributed Runtime/Editor batch.

## Remaining acceptance

The owner-attributed Windows Runtime/Editor lane must compile the current source, execute the lower
regression and ignored marker, and report the plan-specific allocation and latency gates.
