---
title: Runtime07 Exact Event Catalog Namespace
category: zircon_runtime
report_id: Runtime07-exact-event-catalog-namespace-2026-08-28
date: 2026-08-28
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Exact Event Catalog Namespace

## Scope

This slice removes formatter growth from the extension registry's shared `plugin_id.events`
catalog namespace. It preserves module-name parsing, empty plugin-ID rejection, event manifest
validation, and the namespace consumed by event catalog registration.

## Change

- Build the event catalog namespace with exact `plugin_id + ".events"` capacity and append both
  borrowed parts once.
- Preserve the existing first-module-segment identity and invalid-empty-owner behavior.
- Add a Rust exact-output regression and a Python contract covering construction and boundary
  semantics.

## Deterministic Performance Evidence

The standalone optimized Rust model cycles five representative module names, including an empty
owner, short identities, and multi-segment identities, across 65,536 namespace constructions per
sample. It alternates legacy and optimized order across 31 samples, counts allocator calls and
requested bytes inside namespace construction, and asserts exact output equality for every module
name. Both paths produced checksum `29205987332`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 104,858 | 52,429 | 50.000% |
| Requested allocation bytes | 1,730,148 | 891,290 | 48.485% |
| Namespace construction P50 | 15.7331 ms | 6.5440 ms | 58.406% |
| Namespace construction P95 | 36.3057 ms | 17.7495 ms | 51.111% |

Evidence marker: `RUNTIME07_EXACT_EVENT_CATALOG_NAMESPACE_MODEL_V1`.

A second complete run remained favorable: P50 improved 57.399% and P95 improved 49.397%.

## Validation

- `python tools/tests/test_runtime07_exact_event_catalog_namespace_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust model compiled with Rust 1.94.1, asserted exact namespaces for all module
  identities, and passed two complete 31-sample runs.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot freeze.
- Managed Rust compilation and focused tests remain pending in a later asynchronous Runtime07
  batch paired with another completed optimization slice.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
