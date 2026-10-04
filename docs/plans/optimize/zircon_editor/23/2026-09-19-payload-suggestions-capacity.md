---
title: Editor payload suggestions capacity
category: zircon_editor
report_id: Editor836-payload-suggestions-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor836 · payload suggestions capacity

## Scope

The binding payload suggestion path already borrowed the selected root value,
but its array and table projections still grew temporary vectors from zero
capacity. This slice reserves the known output bounds without changing the
borrowed-root, duplicate-key, ordering, append-index, or owned-value contract.

## Implementation

- Reserve `entries.len() + 1` for array suggestions when a template append is
  possible, then extend the indexed entries directly.
- Reserve `entries.len()` for table keys before sorting and projecting owned
  values.
- Add a lower source regression and ignored
  `EDITOR836_PAYLOAD_SUGGESTIONS_CAPACITY_BENCH_V1` marker.

## Deterministic work model

For 4,096 entries, the old zero-capacity array collector models `12` growth
events for 4,097 output records and the table-key collector models `11` for
4,096 keys. The two bounded reservations model `0` and `0` events. This is
allocation-shape evidence only; it is not a claim about allocator, CPU, RSS,
or product p50/p95/p99 performance.

## TDD and local evidence

The source/model contract was intentionally RED against the zero-capacity
collectors and became GREEN after the reservations and direct extensions
(`3/3`). Exact-file Rustfmt passes. The merged Runtime/Editor
capacity/projection receipt covers `172` files and passes `635/635` tests in
`5.247s`; the broader non-tooling `test_*contract.py` batch covers `826`
files and passes `3352/3352` in `38.825s`, with zero failures, errors, load
errors, or skips. Managed Cargo/Windows Release and Editor product percentile
evidence remain pending.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/asset_editor/binding/payload_suggestions.rs` | `C9EA7A25EECA5FACA4F8BBBC6BF0E8E639611EF8FEA8F7F358C6384E10C215E4` |
| `zircon_editor/src/ui/asset_editor/binding/payload_suggestions/borrowed_root_tests.rs` | `4492EA56C058AC5C69759973A4E74486897BB1C7D63B2EE7F24395A1C7517CEF` |
| `tools/tests/test_editor_payload_suggestions_capacity_performance_contract.py` | `D579B475363C84591F13D6FF2DA50A3ED888A850872365EE9B21A3C9F95B5834` |

## Managed acceptance gate

Keep this record at `managed_validation_pending` until the owner-attributed
batched Windows Release lane proves payload suggestion output parity,
allocation behavior, and the declared Editor payload p50/p95/p99 gates.
