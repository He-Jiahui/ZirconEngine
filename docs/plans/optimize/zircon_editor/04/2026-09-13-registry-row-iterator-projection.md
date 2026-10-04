---
title: Editor04 Registry Row Iterator Projection
category: zircon_editor
report_id: Editor04-registry-row-iterator-projection-2026-09-13
date: 2026-09-13
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor04 Registry Row Iterator Projection

## Scope

`EditorAssetIndex::rows` immediately transforms every authoritative Runtime
registry entry into an editor row. Calling `entries()` first creates an owned
temporary vector and then allocates the final row vector, even though the
registry already offers an exact-size canonical iterator.

## Change

- Reserve the final row vector from the Runtime registry length.
- Extend it directly from `entries_iter()` so no intermediate entry vector is
  materialized.
- Preserve row ordering, metadata lookup, dirty/importing state, and Runtime
  registry authority.

## Complexity boundary

Row projection remains `O(N)` but removes one temporary `O(N)` pointer-vector
allocation/copy and retains a single bounded output allocation. This does not
claim incremental catalog projection or product asset-browser p50/p95/p99
acceptance.

## TDD and local evidence

- The source contract was run RED before implementation because `rows()` used
  `entries().into_iter()` and had no output capacity reservation.
- After implementation the iterator/projection contract passes `2/2`.
- The combined Runtime UI plus Editor asset contract discovery passes
  `620/620` in `15.422s`; the focused index/input/asset slice passes `37/37` in
  `0.039s`.
- The broader Runtime/Editor performance-contract and pressure discovery passes
  `1358/1358` in `9.465s` in one process.
- Scoped Python compilation, Rustfmt, `git diff --check`, and wiki validation
  pass.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/asset/index.rs` | `30D345B38D78A9E9344622E3AD7AB760475BC8EAA0A6BB71423E67D21FCD7A76` |
| `tools/tests/test_runtime_editor_registry_iterator_projection_performance_contract.py` | `08F57C9E8D68F85D410C28F7A459CF2C6DA7EC409D9335001411BAD6127A4E7E` |

## Managed gate

This slice joins the existing owner-attributed Runtime/Editor batch. Managed
Cargo/Release allocation and Editor row/catalog p50/p95/p99 evidence remain
pending; no standalone Cargo process or coordinator polling is required.
