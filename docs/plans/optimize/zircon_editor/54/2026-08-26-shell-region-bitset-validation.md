---
title: Editor54 Shell Region Bitset Validation
category: zircon_editor
report_id: Editor54-shell-region-bitset-validation-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor54 Shell Region Bitset Validation

## Scope

This slice replaces shell-region asset completeness validation's temporary `BTreeSet` with a
six-bit `u8` mask. `EditorRegion::ALL` remains the completeness and missing-error order authority,
while input order still determines which duplicate region is rejected first.

TOML parsing, role validation, `RegionBinding` order, skeleton publication, and serialized asset
shape are unchanged.

## Performance Workload

The release workload validates all six regions 100,000 times per sample, including duplicate
admission checks and complete-set membership checks.

| Work per sample | Before | After |
|---|---:|---:|
| Ordered tree instances | 100,000 | 0 |
| Ordered insert/contains probes | 1,200,000 | 0 |
| Region bit tests/sets | 0 | 1,200,000 |
| Region output reordering | 0 | 0 |

The ignored release gate runs 21 alternating sample pairs and emits
`EDITOR54_SHELL_REGION_BITSET_VALIDATION_BENCH_V1`. Acceptance requires bitset validation P95 to
be at least 30% below the legacy `BTreeSet` path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826cl_editor_shell_region_bitset_preserves_duplicate_and_missing_errors`
  covers the real asset plus duplicate-first and ordered missing-region errors.
- `optimization_batch_20260826cl_editor_shell_region_validation_uses_fixed_bitset` locks the six
  explicit region bits and prevents ordered allocation from returning.
- `optimization_batch_20260826cl_editor_shell_region_bitset_release_benchmark` reports paired
  release P50/P95 samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor54 still owns constraint language semantics, responsive region binding, incremental layout,
DPI behavior, publication currentness, and product-scale resize qualification. This slice only
removes ordered allocation from fixed shell-region asset validation.
