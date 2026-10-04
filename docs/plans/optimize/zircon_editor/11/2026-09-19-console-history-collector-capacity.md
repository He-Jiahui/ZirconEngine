---
title: Editor11 console history collector capacity
category: zircon_editor
report_id: Editor807-console-history-collector-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor807 · console history collector capacity

## Scope

`EditorConsoleHistory` already bounds retained logical lines, but three
collectors still started at zero capacity: the entered logical-line batch, the
filtered visible append, and the full filtered snapshot created by
`set_filter`. The bounded source shapes make those upper bounds available
without changing the diagnostic authority, line identity, filter semantics, or
retention policy.

## Implementation

- `push_with_level` reuses the retained logical-line count returned by the
  clipping helper and reserves that count before extending the entered-line
  vector, avoiding a second line-count scan.
- `publish_visible_append` reserves `retained_entered.len()` before the
  filtered/cloned visible append is materialized.
- `set_filter` reserves `self.lines.len()` before rebuilding the filtered
  visible generation; the `All` path still shares the retained generation.
- Existing empty-message and unchanged-filter fast paths remain ahead of
  collection work. `append_bounded`, source/slot identity, level counts,
  overflow expiry, and output deltas are unchanged.

No logging authority, journal routing, UI projection ownership, or product
behavior was changed. The parent Editor11 review remains an architecture and
acceptance ledger; this is a narrow allocation-shape slice.

## TDD and deterministic model

The Python source/model contract was run RED against the prior zero-capacity
collector shape and GREEN after the reservations were added. The lower Rust
source regression checks all three bounded reservation sites. For a 256-line
model, the previous geometric collectors report growth events while the
bounded collectors report zero modeled growth events. Empty-message and
filtered/no-op branches do not reserve additional slots.

This is allocation-shape evidence only; it is not allocator, RSS, CPU, paint
latency, or product p50/p95/p99 evidence.

## Local evidence

- Focused source/model contract:
  `tools/tests/test_editor_console_history_capacity_performance_contract.py`
  (`4/4`).
- Lower source regression:
  `console_history_collectors_reserve_known_line_bounds`.
- Exact-file Rustfmt passes for the production and lower-test files.
- One merged non-tooling Runtime/Editor contract invocation loaded `874`
  files and passed `3681/3681` tests with zero failures, errors, or skips in
  `127.815s`; managed Cargo/Release and product percentile evidence remain
  pending.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/workbench/state/console_history.rs` | `8701935DE44152E302D8CE402420CF7F30002B4D2C943DB445BDDE35D3281FA8` |
| `zircon_editor/src/ui/workbench/state/console_history/tests.rs` | `8D7F783E56F42576F865DF00271909288EB6F74381222CB3B5EA4633DEA2F04D` |
| `tools/tests/test_editor_console_history_capacity_performance_contract.py` | `76BAAF4C10ED412F27D9474B487DA28B51BA64B805AC7EE15B7F0CF7E7459A33` |

## Acceptance boundary

Keep this record `implementation_complete` /
`managed_validation_pending` until the owner-attributed Windows Release batch
compiles the current Editor tree, runs the lower regression, and supplies
allocation plus product Console latency evidence. Tooling production remains
deferred for the later Rust migration.
