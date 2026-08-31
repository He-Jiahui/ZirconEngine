---
title: Runtime07 Exact Runtime Owner Key
category: zircon_runtime
report_id: Runtime07-exact-runtime-owner-key-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Exact Runtime Owner Key

## Scope

This slice removes formatter growth from the extension registry's shared `plugin_id.runtime` owner
identity and routes runtime module registration through the same owner helper. It preserves module
validation, duplicate rejection, interned module identity, module registration, and namespaced-key
ownership.

## Change

- Build the runtime owner with exact `plugin_id + ".runtime"` capacity and append both borrowed
  parts once.
- Reuse `intern_runtime_owner` from runtime module registration instead of maintaining a separate
  formatter path.
- Add a Rust exact-output regression and a Python contract covering key construction and owner-path
  convergence.

## Deterministic Performance Evidence

The standalone optimized Rust model cycles five plugin IDs, including empty, short, and long
identities, across 65,536 owner-key constructions per sample. It alternates legacy and optimized
order across 31 samples, counts allocator calls and requested bytes inside key construction, and
asserts exact output equality for every identity. Both paths produced checksum `43809465944`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 117,964 | 65,536 | 44.444% |
| Requested allocation bytes | 2,660,729 | 1,336,922 | 49.754% |
| Owner-key construction P50 | 14.2556 ms | 5.2551 ms | 63.137% |
| Owner-key construction P95 | 22.7825 ms | 7.8532 ms | 65.530% |

Evidence marker: `RUNTIME07_EXACT_RUNTIME_OWNER_KEY_MODEL_V1`.

A second complete run remained favorable: P50 improved 61.819% and P95 improved 48.554%.

## Validation

- `python tools/tests/test_runtime07_exact_runtime_owner_key_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model compiled with Rust 1.94.1, asserted exact owner keys for all identities,
  and passed two complete 31-sample runs.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot freeze.
- Managed Rust compilation and focused tests remain pending in an asynchronous Runtime07 batch
  paired with the exact interface import key slice.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
