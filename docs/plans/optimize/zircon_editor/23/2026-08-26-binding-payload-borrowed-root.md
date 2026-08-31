---
title: Editor23 Binding Payload Borrowed Root
category: zircon_editor
report_id: Editor23-binding-payload-borrowed-root-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Binding Payload Borrowed Root

## Scope

This slice removes full root-suggestion table projection from contextual binding payload lookup.
Path parsing, nested array/table lookup, duplicate root-key last-wins behavior, output ordering,
append-index selection, and returned value ownership remain unchanged.

## Change

- Split the parsed path into its root key and remaining segments.
- Reverse-search the borrowed root suggestion slice so duplicate keys preserve the old table
  collection's last-wins semantics.
- Pass the borrowed root value and remaining segments to the existing path resolver instead of
  cloning all root keys and nested values into a temporary TOML table.

## Deterministic Performance Evidence

| 4,096 roots, four contextual builds per sample | Before | After |
|---|---:|---:|
| Temporary root tables | 4 | 0 |
| Root entries deeply cloned | 16,384 | 0 |
| Full nested root values deeply cloned | 16,384 | 0 |
| Borrowed root-key comparisons | 0 | up to 16,384 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR23_PAYLOAD_SUGGESTIONS_BORROWED_ROOT_BENCH_V1`. Acceptance requires borrowed-root lookup P95
to be at least 50% below full root projection. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826at_payload_suggestions_preserve_nested_last_root` covers nested
  arrays, append index, duplicate root keys, and invalid root-index paths.
- `optimization_batch_20260826at_payload_suggestions_borrow_root_values` rejects temporary TOML
  root projection and requires reverse borrowed lookup plus the existing tail resolver.
- `optimization_batch_20260826at_payload_suggestions_borrowed_root_p95` reports paired P50/P95
  samples and enforces the 50% P95 reduction gate.

## Remaining Parent-plan Work

Editor23 still owns schema-backed preview data, typed diagnostics, incremental validation, preview
fidelity, bindings, transactions, cook artifacts, and large-asset gates. This slice only converges
contextual binding payload suggestion lookup.
