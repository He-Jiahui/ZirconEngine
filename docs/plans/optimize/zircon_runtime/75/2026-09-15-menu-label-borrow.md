---
title: Runtime75 Menu Label Borrow
category: zircon_runtime
report_id: Runtime75-menu-label-borrow-2026-09-15
date: 2026-09-15
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Menu Label Borrow

## Scope

Menu search-tree construction cloned a map label into an intermediate `String` before cloning it
again into each retained option. The intermediate ownership was unnecessary for read-only label
selection.

## Implementation

- `menu_option_label_text` now returns a borrowed `&str` selected from the existing map value.
- Retained option nodes clone the label only at the ownership boundary; ID fallback behavior is
  unchanged for missing or empty labels.
- Added lower multi-ID/label-fallback regression and ignored marker
  `RUNTIME780_MENU_LABEL_BORROW_BENCH_V1`.

## Deterministic work model

Per map node, label lookup no longer allocates a temporary owned string before the retained node
copy. This is allocation-shape evidence only, not a claim about allocator, CPU/RSS, or product menu
latency percentiles.

## Validation

- TDD source contract:
  `tools/tests/test_runtime_menu_label_borrow_performance_contract.py` was RED before
  implementation and is GREEN at `3/3`.
- The combined Runtime/Editor source-contract batch passes `120/120` in `0.080s`.
- `rustfmt --edition 2021 --check` passes for the production menu reducer and lower regression.
- Managed Cargo, Release allocation, and product p50/p95/p99 evidence remain pending in the
  owner-attributed Runtime/Editor batch.

## Remaining acceptance

The owner-attributed Windows Runtime/Editor lane must compile the current source, execute the
lower regression and ignored marker, and report the plan-specific allocation and latency gates.
