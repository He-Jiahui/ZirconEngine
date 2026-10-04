---
title: Runtime75 Surface Tree Interaction Shared Catalog
category: zircon_runtime
report_id: Runtime75-surface-tree-interaction-shared-catalog-2026-09-15
date: 2026-09-15
session_id: root-astra-optimize-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime75 Surface Tree Interaction Shared Catalog

## Scope

The v2 surface-tree interaction inference path kept a private `OnceLock` containing an owned
`editor_showcase()` registry. The catalog is immutable and already has a process-wide shared view,
so every interaction inference process could otherwise retain a second 69-descriptor catalog.

## Implementation

- `component_descriptor_registry` now returns
  `UiComponentDescriptorRegistry::editor_showcase_shared()` directly.
- Removed the redundant local `OnceLock`; interaction capability lookup retains the same descriptor,
  category, and event semantics.
- Added a pointer-identity lower regression and ignored release marker
  `RUNTIME782_SURFACE_TREE_SHARED_CATALOG_BENCH_V1`.

## Deterministic work model

The old path materialized one owned 69-descriptor registry on first surface-tree interaction
inference. The optimized path retains zero secondary descriptor clones and borrows the shared
catalog. This is allocation-shape evidence only, not a claim about allocator, CPU/RSS, or product
interaction latency percentiles.

## Validation

- TDD source contract:
  `tools/tests/test_runtime_surface_tree_interaction_shared_catalog_performance_contract.py` was
  RED before implementation and is GREEN at `3/3`.
- The current-source Runtime/Editor contract loader passes `1948/1948` across `543` modules, and
  the focused Runtime/Editor hot-path set passes `142/142` in one process. These are local
  source/model receipts only.
- `rustfmt --edition 2021 --check` passes for the production interaction module and lower regression.
- Managed Cargo, Release allocation, and product p50/p95/p99 evidence remain pending in the
  owner-attributed Runtime/Editor batch.

## Remaining acceptance

The owner-attributed Windows Runtime/Editor lane must compile the current source, execute the lower
regression and ignored marker, and report the plan-specific allocation and latency gates.
