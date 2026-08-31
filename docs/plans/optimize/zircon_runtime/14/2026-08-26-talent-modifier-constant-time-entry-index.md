---
title: Runtime14 Talent Modifier Constant-Time Entry Index
category: zircon_runtime
report_id: Runtime14-talent-modifier-constant-time-entry-index-2026-08-26
date: 2026-08-26
session_id: root-runtime14-talent-entry-index-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime14 talent modifier constant-time entry index

## Scope

- Parent gaps: the entry-lookup portion of `WOC-PROG-P1-051` and `WOC-PROG-P1-058`.
- Baseline: `8e56165c4c789416c328898d3d8937d934b52efa`, epoch `443`.
- Owners: the talent modifier generator, its generated Zr catalog, the reachable talent modifier reducer, source/performance contract, and this record.
- This slice removes only the 189-row selection-to-entry lookup scan. Allocation-revision snapshot caching, WorldState projection allocation, modifier application field ladders, ability/string lookup, proc/effect compilation, and complete Runtime14 qualification remain open.

## Change

- The generator now validates the current dense layout before relying on it: each of nine classes owns exactly three spec entries followed by eighteen option entries, and both code domains are contiguous in class/source order.
- The generated catalog exposes `entryIndex(originCode, spec)`, validates zero and upper bounds, and maps a valid code to the generated row using fixed division, multiplication, addition, and remainder operations.
- `talent_modifier_state.findEntry` delegates directly to the generated lookup and no longer calls `entryOriginCode` / `entryIsSpec` while scanning up to 189 rows.
- The Zr contract round-trips all 189 generated entries through `entryIndex` and checks invalid zero, spec 28, and option 163 inputs return `-1`.
- The reference JSON and its catalog digest remain unchanged; only the generator-owned Zr projection gains the indexed accessor.

Worst-case row comparisons fall from 189 to three fixed admission/origin decisions plus integer arithmetic, a 98.413% structural reduction. A full seven-selection recompute no longer multiplies selection count by catalog row count for entry discovery.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_runtime14_talent_modifier_entry_lookup_performance_contract -v` initially passed 3/7 and failed the four guards requiring layout validation, generated constant-time lookup, reducer delegation, and Zr exhaustive coverage.
- GREEN: the same command passes 7/7 after generation and consumer changes.
- `node examples/woc/tools/talent_modifier_catalog_codegen.mjs --check` passes for all 189 entries.
- `node --check examples/woc/tools/talent_modifier_catalog_codegen.mjs` and the private benchmark syntax check pass.
- Scoped `git diff --check` passes apart from Git's existing LF/CRLF checkout notices.

The deterministic Node model validates all 189 entries and three invalid cases, then measures 31 alternating legacy/optimized sample pairs over 250,000 fixed pseudo-random spec/option queries per sample using nearest-rank percentiles and observable row-inspection counters.

| Metric | 189-row scan | Generated integer index | Change |
|---|---:|---:|---:|
| P50 | 93.2000 ms | 5.0688 ms | -94.561% |
| P95 | 162.0843 ms | 7.8977 ms | -95.127% |
| row inspections / fixed decisions per 250k queries | 23,720,250 | 750,000 | -96.838% |

These timings isolate selection-code-to-modifier-entry lookup. They do not claim end-to-end talent recompute, combat tick, ZrVM startup, modifier field application, or snapshot-cache latency.

## Async validation

One coordinator batch must run the seven Python contracts, generator syntax/freshness checks, the parity/performance model, scoped diff checks, and `woc_m5_talent_modifier_state_tests.zrp` against pinned external ZrVM commit `60f6bcf4dd22bb6f5247e353bd0d97964758f157` in one managed Cargo group.

Acceptance requires the Zr package to compile and return zero, all 189 entries and three invalid cases to retain parity, generator freshness to pass, and P50/P95 reductions to remain at least 35%. Integration and automatic WeCom publication remain coordinator-owned after managed validation and independent review succeed. The WeCom message must include the managed P50/P95 and row-inspection result and label it as talent modifier entry-lookup-only evidence.
