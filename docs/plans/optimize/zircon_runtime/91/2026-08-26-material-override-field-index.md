---
title: Runtime91 Material Override Field Index
category: zircon_runtime
report_id: Runtime91-material-override-field-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime91 Material Override Field Index

## Scope

This slice replaces repeated material uniform layout scans during override application with one
borrowed field index. Payload cloning, override order, duplicate-name first-field behavior,
unknown-property/type/layout diagnostics, encoded bytes, and returned ownership remain unchanged.

## Change

- Build one `&str -> first layout index` HashMap after cloning the output payload.
- Preserve the old first-match rule with entry `or_insert` when duplicate field names exist.
- Resolve every override through the borrowed index before parsing and encoding its field kind.

## Deterministic Performance Evidence

| 2,048 fields and overrides, four applications per sample | Before | After |
|---|---:|---:|
| Field-name comparisons per sample | 8,392,704 | 0 |
| Field index construction visits per sample | 0 | 8,192 |
| Field hash lookups per sample | 0 | 8,192 |
| Payload output clones per sample | 4 | 4 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME91_MATERIAL_OVERRIDE_FIELD_INDEX_BENCH_V1`. Acceptance requires indexed override P95 to be
at least 90% below repeated layout scans. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `runtime91_material_override_index_preserves_first_field_and_diagnostics`
  covers duplicate-name first match, type mismatch, unknown property, and encoded bytes.
- `runtime91_material_override_uses_borrowed_field_index` requires borrowed
  HashMap lookup with first-entry preservation and rejects linear field search.
- `runtime91_material_override_field_index_p95` reports paired P50/P95 samples
  and enforces the 90% P95 reduction gate.

The three tests are grouped with shader-import provider deduplication and shared include analysis in
one three-task asynchronous coordinator batch. Terminal timings, integration, record finalization,
and automatic WeCom delivery remain pending.

## Remaining Parent-plan Work

Runtime91 still owns shader modules, permutations, reflection, layouts, pipelines, PSO caching,
prewarm, hot reload, and product-scale receipts. This slice only converges runtime uniform override
lookup.
