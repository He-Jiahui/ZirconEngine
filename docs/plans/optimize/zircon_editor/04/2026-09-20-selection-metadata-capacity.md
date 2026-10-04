---
title: Editor848 Asset Selection Metadata Capacity
category: zircon_editor
report_id: Editor848-selection-metadata-capacity-2026-09-20
date: 2026-09-20
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor848 - asset selection metadata capacity

## Scope

The Asset Browser selection-detail projection rebuilds a short summary and a
line-oriented metadata body on every selection/detail refresh. Both temporary
vectors previously started with zero capacity even though the emitted shape is
known from the immutable selection snapshot.

## Optimization

- Reserve the summary vector from its fixed toolkit entry plus the four
  optional metadata fields.
- Reserve the body vector from its mandatory diagnostics line and the exact
  included-file/subasset section lengths, using saturating arithmetic.
- Keep text, ordering, empty-section behavior, and downstream `join` semantics
  unchanged; this slice only changes temporary vector growth.

## TDD and deterministic evidence

The Python source/model contract was intentionally run RED before the capacity
helpers and reservations existed, then GREEN after production and lower wiring
were added (`4/4`). The folder-backed lower module covers the maximum summary
shape, body section cardinality, and the ignored
`EDITOR848_SELECTION_METADATA_SUMMARY_CAPACITY_BENCH_V1` Release marker. The
deterministic five-entry summary and seven-line body model removes geometric
growth (`2 -> 0` and `2 -> 0`, respectively).

## Local validation

- `tools/tests/test_editor_selection_metadata_capacity_performance_contract.py`:
  `4/4`.
- Exact-file Rustfmt and Python compilation pass for the production, lower, and
  contract files.
- The focused seven-contract Runtime/Editor loader passes `28/28` tests in
  `0.027s`; the broad non-tooling performance/pressure loader passes
  `2386/2386` tests across `652` modules in `14.287s`, with zero load errors,
  failures, errors, or skips. Managed Windows Cargo/Release, allocator, and
  Asset Browser product p50/p95/p99 evidence remain pending behind the external
  worktree admission gate.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/layouts/views/asset_browser/selection_text.rs` | `FA55102EFF51A34BF9C555E86AA39A594E4D4466FFCFA0874CFB0D0E83FC9D76` |
| `zircon_editor/src/ui/layouts/views/asset_browser/selection_text/capacity_tests.rs` | `AECDB4E19CD9EAF4D6F3AD08A6B05C78986D2F430EFA2D6E13FA0926ECFAA509` |
| `tools/tests/test_editor_selection_metadata_capacity_performance_contract.py` | `5E6D4F52E9ADDAF0AB19EDD078C29137E4CD94E888DD711DE67A70789CB3142D` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed managed Windows Release lane compiles the current
Runtime/Editor tree, executes the lower regression and ignored marker, and
supplies allocator plus Asset Browser product p50/p95/p99 evidence. Tooling
production remains deferred for the later Rust migration.
