---
title: Editor50 Plugin Manifest Package Index
category: zircon_editor
report_id: Editor50-plugin-manifest-package-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor50 Plugin Manifest Package Index

## Scope

This slice removes two package-by-selection Cartesian scans from built-in Editor plugin manifest
completion. Runtime catalog completion, original selection order, missing package append order,
explicit editor crate overrides, first matching package semantics, and projected package defaults
remain unchanged. It advances Editor50 plugin catalog projection without claiming completion of
extension lifecycle, unload, native parity, conflict policy, or product integration gates.

## Change

- Build one first-package-wins map from package ID to Editor package manifest.
- Mark project-present package IDs in a borrowed presence set.
- Append missing package selections in original catalog order and suppress duplicate package IDs.
- Fill missing editor crate names through direct package-ID lookups.

## Deterministic Performance Evidence

| 1,024 disjoint project selections and 1,024 Editor packages | Before | After |
|---|---:|---:|
| Pairwise package/selection comparisons | 2,621,952 | 0 |
| Package index-build visits | 0 | 1,024 |
| Selection package-map lookups | 0 | 2,048 |
| Package presence probes | 0 | 1,024 |
| Selection or catalog order changes | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR50_PLUGIN_MANIFEST_PACKAGE_INDEX_BENCH_V1`. Acceptance requires indexed completion P95 to be
at least 75% below the legacy two-scan completion. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826be_plugin_manifest_package_index_preserves_completion_order`
  covers original order, missing append order, explicit crate preservation, and indexed crate fill.
- `optimization_batch_20260826be_plugin_manifest_package_index_eliminates_pairwise_work` locks the
  2,621,952-comparison model and rejects package/selection `.any` and `.find` scans.
- `optimization_batch_20260826be_plugin_manifest_package_index_p95` reports paired release P50/P95
  samples and enforces the 75% P95 reduction gate.

## Remaining Parent-plan Work

Editor50 still owns extension contribution generations, conflict/finalization policy, provider
selection, native/static parity, reload/unload leases, retained surface integration, and product
fault/soak evidence. This slice only converges built-in project manifest completion.
