---
title: Runtime09C Material Option Value Hash Index
category: zircon_runtime
report_id: Runtime09C-material-option-value-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime09c-three-task-material-index-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09C Material Option Value Hash Index

## Scope

This slice adds a borrowed name index while resolving values for large material option tables.
Tables with fewer than 16 options or fewer than two supplied values retain the allocation-free
linear path. Larger tables build one bounded `HashMap<&str, &MaterialOptionRef>` for the call and
then resolve every supplied value without rescanning the option vector.

Value application still folds the caller's ordered `BTreeMap`, preserving overwrite order even
for malformed overlapping bit ranges. Index admission keeps the first duplicate option name,
matching the former linear lookup. Default bits, unknown values, invalid value types, option
serialization, definition ordering, and bit packing are unchanged.

## Deterministic Work Model

The release workload resolves 512 boolean options 16 times. Each call supplies every option.

| Work per workload | Before | After |
|---|---:|---:|
| Linear option-name comparisons | 2,101,248 | 0 |
| Borrowed hash-index insertions | 0 | 8,192 |
| Hash option lookups | 0 | 8,192 |
| Owned key allocations | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`RUNTIME09C_MATERIAL_OPTION_VALUE_HASH_INDEX_BENCH_V1`. Acceptance requires indexed resolution P95
to be at least 80% below repeated linear scans. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `runtime09c_batch_material_option_hash_index_preserves_value_order` covers ordered
  overwrite behavior, duplicate-name ownership, unknown values, and parity with the legacy scan.
- `runtime09c_batch_material_option_hash_index_keeps_small_table_fast_path` locks the
  allocation-free threshold and borrowed hash-index structure.
- `runtime09c_batch_material_option_hash_index_p95` reports paired release P50/P95
  samples and enforces the 80% P95 reduction gate.
- The managed `runtime09c_batch_` release gate covers this task, material-property schema rescan
  elision, and shading-token hashing in one Cargo invocation: 3 source contracts, 9 Rust tests,
  and 3 performance rows. Dynamic marker values, integration commit, and WeCom delivery remain
  coordinator-owned and pending.

## Remaining Parent-plan Work

Runtime09C still owns shader compilation, material variant policy, pipeline state lifetime,
persistence, and product GPU evidence. This slice only converges material option value resolution.
