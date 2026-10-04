---
title: Runtime75 Menu Child Values Iterator
category: zircon_runtime
report_id: Runtime75-menu-child-values-iterator-2026-09-15
date: 2026-09-15
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Menu Child Values Iterator

## Scope

Menu search-tree construction first collected present child-property values into a temporary
`Vec<&UiValue>` for every map node. The property set is fixed and small, so that borrowed vector
was avoidable.

## Implementation

- `menu_child_values` now returns a lazy iterator over the canonical child-property name array.
- Child traversal consumes the iterator directly, preserving the established property precedence
  (`children`, `items`, `submenu`, `sub_menu`, `subMenu`, `options`) without an intermediate borrow
  vector.
- The older menu-search streaming contract was updated to encode the new no-vector boundary.
- Added lower property-order/no-allocation regression and ignored marker
  `RUNTIME781_MENU_CHILD_VALUES_ITERATOR_BENCH_V1`.

## Deterministic work model

Each visited map node removes one temporary child-value vector and its allocation metadata while
retaining the output `MenuSearchOption` vector. This is allocation-shape evidence only, not a
claim about allocator, CPU/RSS, or product menu-search latency percentiles.

## Validation

- TDD source contract:
  `tools/tests/test_runtime_menu_child_values_iterator_performance_contract.py` was RED before
  implementation and is GREEN at `3/3`.
- The combined Runtime/Editor source-contract batch passes `120/120` in `0.080s`.
- `rustfmt --edition 2021 --check` passes for the production menu reducer and lower regression.
- Managed Cargo, Release allocation, and product p50/p95/p99 evidence remain pending in the
  owner-attributed Runtime/Editor batch.

## Remaining acceptance

The owner-attributed Windows Runtime/Editor lane must compile the current source, execute the
lower regression and ignored marker, and report the plan-specific allocation and latency gates.
