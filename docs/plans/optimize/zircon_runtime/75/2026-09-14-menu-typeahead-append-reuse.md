---
title: Runtime75 Menu Typeahead Append Reuse
category: zircon_runtime
report_id: Runtime75-menu-typeahead-append-reuse-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Menu Typeahead Append Reuse

## Scope

When a menu typeahead buffer was active, its normalized temporary `String` was interpolated with
the new payload through `format!`, forcing a separate combined string before repeat-key and
fallback-search decisions. The temporary prior buffer is already uniquely owned at that point.

## Implementation

- `menu_typeahead_searches` moves the prior normalized buffer into `combined` and appends the
  current payload in place.
- Existing capacity is reused when it can contain the next scalar; a reallocation remains possible
  only when the expanded result genuinely needs more room.
- Active, expired, repeated-key, primary search, fallback single-key search, and current-preference
  semantics remain unchanged.
- Added a lower state-transition regression and ignored marker
  `RUNTIME776_MENU_TYPEAHEAD_APPEND_REUSE_BENCH_V1`.

## Deterministic work model

For a retained prior buffer with sufficient capacity, appending eliminates the separately allocated
combined string. The returned search publication still owns its required values. This is
allocation-shape evidence, not a claim about allocator, CPU/RSS, or product keyboard latency
percentiles.

## Validation

- TDD source contract: `tools/tests/test_runtime_menu_typeahead_append_reuse_performance_contract.py`
  was RED before implementation and is GREEN at `3/3`.
- Focused menu/typeahead/search source-contract batch passes `18/18` in `0.041s`.
- The current combined Runtime/Editor source-contract batch passes `120/120` in `0.080s`.
- `rustfmt --edition 2021 --check` passes for the production menu reducer and lower regression.
- Managed Cargo, Release allocation, and product p50/p95/p99 evidence remain pending in the
  owner-attributed Runtime/Editor batch.

## Remaining acceptance

The owner-attributed Windows Runtime/Editor lane must compile the current source, execute the
lower regression and ignored marker, and report the plan-specific allocation and latency gates.
