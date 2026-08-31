---
title: Runtime07 Streamed Component Entity IDs
category: zircon_runtime
report_id: Runtime07-streamed-component-entity-ids-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Streamed Component Entity IDs

## Scope

This slice removes the second owned entity vector from the guest `find_by_component` host call.
The World still produces the same stable, component-scoped row projection; only the redundant
`rows -> Vec<EntityId>` step before JSON serialization is removed.

## Change

- Add a borrowed serde sequence view over the entity column of dynamic component rows.
- Serialize entity IDs directly from the row slice without cloning component values or allocating
  another vector.
- Preserve row order, numeric entity JSON representation, error propagation, and the returned
  script string.
- Add Rust exact-output coverage and a Python source contract that rejects reintroducing the
  second entity vector.

## Deterministic Performance Evidence

The standalone optimized Rust model isolates the entity projection layer shared by the serde
sequence traversal. Each sample performs 2,048 projections over 4,096 stable rows, alternates
legacy and optimized execution order across 31 samples, counts allocator calls and requested bytes
for one projection, and verifies identical rolling checksums. It deliberately excludes World row
collection and JSON output-buffer costs because those are unchanged by this slice.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Entity projection allocation calls | 1 | 0 | 100% |
| Entity projection requested bytes | 32,768 | 0 | 100% |
| Entity projection P50 | 19.5016 ms | 14.4315 ms | 25.998% |
| Entity projection P95 | 45.7053 ms | 24.0406 ms | 47.401% |

Evidence marker: `RUNTIME07_STREAMED_COMPONENT_ENTITY_IDS_MODEL_V1`.

A second complete run remained favorable: P50 improved 28.026% and P95 improved 25.358%.
Both paths produced checksum `16119968421641282560`.

## Validation

- `python tools/tests/test_runtime07_streamed_component_entity_ids_performance_contract.py`:
  3 passed after the pre-change contract failed 3 of 3 checks.
- The standalone Rust 1.94.1 model compiled and passed two complete 31-sample runs with identical
  checksums and zero optimized projection allocations.
- The Rust unit guard asserts exact `[7,11]` output while ignoring component values.
- Exact-file Rust formatting, Python bytecode compilation, and scoped diff checks are required
  before snapshot freeze.
- Managed Rust compilation and focused tests remain pending in an asynchronous Runtime07 batch.

Managed batch request: `runtime07-borrowed-gameplay-seven-task-batch-20260830-v1`.

Validation attempt: ticket `a9dc9a55e9044c239cc7dfda8bbc64b6` failed before Cargo at
coordinator artifact governance for `D:\ZirconBuilds\mvp-test-fixtures-36724`; the 22 local contract
checks remain green while integrated acceptance and success publication remain pending.

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
