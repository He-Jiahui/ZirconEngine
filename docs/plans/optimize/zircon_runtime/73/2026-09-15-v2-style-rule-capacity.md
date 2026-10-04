---
title: Runtime73 V2 Style Rule Capacity
category: zircon_runtime
report_id: Runtime73-v2-style-rule-capacity-2026-09-15
date: 2026-09-15
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime73 V2 Style Rule Capacity

## Scope

The v2 style resolver flattened every rule from every authored stylesheet into a `Vec` that
started with zero capacity. The stylesheet rule count is available before selector parsing, so
the collector can reserve the exact lower bound without changing parsing, specificity sorting, or
selector error behavior.

## Implementation

- Count `stylesheet.rules.len()` across the document once before rule collection.
- Initialize the resolved-rule vector with `Vec::with_capacity(rule_count)`.
- Preserve global rule order, specificity sorting, cloned declaration blocks, and invalid-selector
  diagnostics.
- Add a lower order-preservation regression and ignored marker
  `RUNTIME785_V2_STYLE_RULE_CAPACITY_BENCH_V1`.

## Deterministic performance model

For `R` authored rules, the old zero-capacity vector grows geometrically while the optimized
collector reserves `R` slots before parsing. The model removes capacity-growth events only; it is
not allocator, CPU, RSS, or product style-latency evidence.

## Validation

- TDD source contract `tools/tests/test_runtime_v2_style_rule_capacity_performance_contract.py`
  was RED before the reservation and is GREEN at `3/3`.
- The pre-785 Runtime/Editor prefix batch passed `1270/1270` across `349` modules in `10.629s`;
  the current non-tooling core batch now passes `1963/1963` across `549` modules in `14.682s`,
  including Runtime785. These are local source/model receipts only.
- The merged recent Runtime/Editor/input/style focused batch passes `316/316` across `61` modules
  in `5.877s`, with zero failures, errors, or skips.
- Scoped Rustfmt passes for the v2 style resolver and lower regression.
- Managed Cargo, Release allocation, and product p50/p95/p99 evidence remain pending.

## Remaining acceptance

The owner-attributed Windows Runtime/Editor lane must compile the current source, execute the lower
regression and ignored marker, and report the plan-specific allocation and latency gates.
