---
title: Runtime75 Menu Typeahead Text Normalization
category: zircon_runtime
report_id: Runtime75-menu-typeahead-text-normalization-2026-09-14
date: 2026-09-14
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Menu Typeahead Text Normalization

## Scope

Every menu typeahead event removed control characters into one temporary `String`, trimmed it,
then allocated a second lowercase `String` that became the retained search buffer. The reducer only
needs the final normalized buffer. This slice builds that output once while retaining the existing
filter, trim, and Unicode lowercase behavior.

## Implementation

- `keyboard_text_search` starts with one final output buffer, preserving no-allocation
  empty/control-only behavior and avoiding retention of raw-input-sized capacity after trimming.
- It skips control characters and leading whitespace, records the start of a trailing whitespace
  suffix, lowercases each retained scalar directly into the final buffer, then truncates a trailing
  suffix once at completion.
- Control-only input, Unicode whitespace, internal whitespace after control removal, and
  multi-scalar lowercase expansion preserve the retired filter/trim/lowercase result.
- Added a lower equivalence regression and ignored marker
  `RUNTIME775_MENU_TYPEAHEAD_TEXT_NORMALIZATION_BENCH_V1`.

## Deterministic work model

Each typeahead event now creates only the required returned normalization buffer instead of first
constructing a filtered intermediate string. It retains constant-size trailing-suffix state. This
is allocation-shape evidence, not a claim about allocator, CPU/RSS, or product keyboard latency
percentiles.

## Validation

- TDD source contract: `tools/tests/test_runtime_menu_typeahead_text_normalization_performance_contract.py`
  was RED before implementation and is GREEN at `3/3`.
- Focused menu/typeahead/search source-contract batch passes `21/21` in `0.092s`.
- The current combined Runtime/Editor source-contract batch passes `120/120` in `0.080s`.
- `rustfmt --edition 2021 --check` passes for the production menu reducer and lower regression.
- Managed Cargo, Release allocation, and product p50/p95/p99 evidence remain pending in the
  owner-attributed Runtime/Editor batch.

## Remaining acceptance

The owner-attributed Windows Runtime/Editor lane must compile the current source, execute the
lower regression and ignored marker, and report the plan-specific allocation and latency gates.
