---
title: Runtime75 Menu Typeahead Search Projection
category: zircon_runtime
report_id: Runtime75-menu-typeahead-search-projection-2026-09-15
date: 2026-09-15
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Menu Typeahead Search Projection

## Scope

Each `MenuTypeaheadSearch` retained two identical owned strings: one for matching and one for
publishing the typeahead buffer. Pasted and Unicode-expanded input also used a complete
`chars().count()` scan merely to decide whether more than one scalar was present.

## Implementation

- Reduced `MenuTypeaheadSearch` to its one required owned `buffer`; matching now borrows that
  same buffer.
- Moves the selected payload/combined buffer directly into each emitted candidate, preserving
  primary-search then fallback-search order without duplicated candidate storage.
- Uses `chars().nth(1).is_some()` for the multi-scalar decision, so that decision stops after the
  second Unicode scalar while retaining expansion behavior such as `U+0130` lowercasing to two
  scalars.
- Preserves pasted input, active-buffer, expired-buffer, repeated-key, current-preference, and
  fallback semantics.
- Added lower regression coverage and ignored marker
  `RUNTIME777_MENU_TYPEAHEAD_SEARCH_PROJECTION_BENCH_V1`.

## Deterministic work model

Each emitted candidate previously carried a duplicate owned search string in addition to its
published buffer. The projection now retains one owned string per candidate. Multi-scalar
detection no longer traverses the input after it has observed the second scalar. This is
allocation-shape and bounded-work evidence only, not a claim about allocator, CPU/RSS, or product
keyboard latency percentiles.

## Validation

- TDD source contract:
  `tools/tests/test_runtime_menu_typeahead_search_projection_performance_contract.py` was RED
  before implementation and is GREEN at `4/4`.
- The combined Runtime/Editor source-contract batch passes `120/120` in `0.080s`.
- `rustfmt --edition 2021 --check` passes for the production menu reducer and both affected lower
  regressions.
- Managed Cargo, Release allocation, and product p50/p95/p99 evidence remain pending in the
  owner-attributed Runtime/Editor batch.

## Remaining acceptance

The owner-attributed Windows Runtime/Editor lane must compile the current source, execute the
lower regressions and ignored marker, and report the plan-specific allocation and latency gates.
