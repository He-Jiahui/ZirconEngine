---
title: Runtime07 derived test inventory M0 snapshot
date: 2026-08-23
plan: docs/plans/optimize/zircon_tooling/32-hot-path-catalog-algorithmic-complexity-data-movement-batching-cache-locality-performance-governance-review.md
status: partial_m0_static_validated_dynamic_pending
scope: runtime07 test inventory count derivation
---

# Runtime07 Test Inventory Derivation

## Change

`performance_hotpath_boundary_audit` now derives `expected_test_file_count` directly from
`RUNTIME_07_TEST_FILES`. The prior independent `91` literal disagreed with the manifest's 90
entries, allowing the report and the declared inventory to drift. The Python regression replaces
the manifest at runtime and proves that the reported count follows the replacement instead of a
fixed value. The legacy count symbol is removed entirely; Runtime07 Rust source-contract tests
anchor the sole `RUNTIME_07_TEST_FILES` manifest instead of any count literal.

## Current M0 Snapshot

The direct static audit after this change reports:

- source inventory: `40/46`; six retired `animation/scene_hook/*` paths are absent;
- test inventory: `90/90`, with the expected count derived from the manifest;
- missing anchors: frame span `2`, query `1`, extract `2`, asset worker `10`, animation `19`;
- large-file gate: `12` hotspots, `4` migration-debt entries, status
  `migration-debt-present`, and missing owner classes `editor-ui` and `support-hub`;
- risks: `7`.

This is a frozen red M0 snapshot, not an accepted performance qualification. The asset-worker
submodule, animation metric handoff, renamed extract/scheduler anchors, typed finding
classification, and all dynamic benchmark/product gates remain owned by their designated plans.

## Validation

- RED: manifest-injection regression returned `1 != 91` before the production change.
- GREEN: the same regression passed after derivation.
- Coordinator static gate: ticket `75914d47c18e41babfaff89442b2ec41` ran
  `tools.tests.test_runtime_performance_hotpath_boundary` against manifest
  `d90ae0f183d104c8e97b3ab7df423816dc1867184b5f6523c8989194ca67e2fc` and passed
  `2` tests in `2.963s` with exit `0`.
- Pending Rust gate: `cargo test -p zircon_runtime runtime_07_performance_hotpath --lib`
  remains required before integration. Its prior coordinator ticket could not materialize because
  the shared baseline is degraded; it has not been treated as a source or test failure.

## Performance Data

No runtime performance measurement or threshold result was produced by this structural audit
change. The only quantitative change is audit correctness: a false expected test count of `91`
now derives as `90` from the current manifest.
