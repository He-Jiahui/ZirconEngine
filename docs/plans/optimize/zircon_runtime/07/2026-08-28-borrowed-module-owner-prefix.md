---
title: Runtime07 Borrowed Module Owner Prefix
category: zircon_runtime
report_id: Runtime07-borrowed-module-owner-prefix-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Borrowed Module Owner Prefix

## Scope

This slice removes the temporary owner-prefix `String` allocated for every runtime plugin module
name validation. It preserves the required `owner_id.` boundary, invalid-name diagnostics, similar
owner rejection, missing-separator rejection, and the existing empty-owner behavior.

## Change

- Match the existing module name against the borrowed owner ID with `strip_prefix` and verify the
  following byte is the namespace separator.
- Keep diagnostic construction on the invalid path unchanged while making valid-name validation
  allocation-free.
- Add a Rust behavior regression for valid, similar-owner, missing-separator, and empty-owner cases,
  plus a Python structure contract that prevents prefix formatting from returning to the hot path.

## Deterministic Performance Evidence

The standalone optimized Rust model validates 65,536 module names per sample, with 14 valid names,
one similar-owner name, and one missing-separator name in every 16 calls. It alternates legacy and
optimized order across 31 samples, counts allocator calls and requested bytes only inside the
predicate, and asserts identical results for every input. Both paths produced checksum
`1879134208`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 131,072 | 0 | 100.000% |
| Requested allocation bytes | 4,915,200 | 0 | 100.000% |
| Prefix validation P50 | 23.1154 ms | 3.7692 ms | 83.690% |
| Prefix validation P95 | 44.3069 ms | 10.2379 ms | 76.890% |

Evidence marker: `RUNTIME07_BORROWED_MODULE_OWNER_PREFIX_MODEL_V1`.

A second complete run remained favorable: P50 improved 81.720% and P95 improved 64.740%.
Invalid-path diagnostic allocation is intentionally excluded from both model paths so the result
isolates the prefix decision changed here.

## Validation

- `python tools/tests/test_runtime07_borrowed_module_owner_prefix_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model compiled with Rust 1.94.1, asserted exact per-name equivalence, and
  passed two complete 31-sample runs.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot freeze.
- Managed Rust compilation and focused tests remain pending in an asynchronous Runtime07 batch
  paired with the borrowed native system-access UTF-8 slice.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
