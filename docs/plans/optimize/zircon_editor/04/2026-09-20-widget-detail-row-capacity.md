---
title: Editor850 Widget Detail Row Capacity
category: zircon_editor
report_id: Editor850-widget-detail-row-capacity-2026-09-20
date: 2026-09-20
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor850 - widget detail-row capacity

## Scope

The retained-host UI Asset Editor builds the widget inspector detail rows on
each pane projection. The output has three fixed fields whose visibility is
known from the presentation and at most six bounded prop/state rows. The
builder previously started at zero capacity and grew while formatting rows.

## Optimization

- Count the three fixed rows using the same empty/force-visible predicate as
  `push_detail_row`.
- Count only actionable prop/state rows within `PROP_STATE_ROW_LIMIT` without
  allocating action IDs during the capacity pass.
- Reserve the exact emitted upper bound while preserving row order, labels,
  action IDs, control suffixes, disabled state, and invalid-row filtering.

## TDD and deterministic evidence

The Python source/model contract was intentionally run RED before the capacity
helper and reservation existed, then GREEN after production and lower wiring
were added (`4/4`). The folder-backed lower module covers the dense nine-row
shape, invalid-row filtering, and the ignored
`EDITOR850_WIDGET_DETAIL_ROW_CAPACITY_BENCH_V1` Release marker. The
deterministic nine-row model removes geometric growth (`3 -> 0`).

## Local validation

- `tools/tests/test_editor_widget_detail_row_capacity_performance_contract.py`:
  `4/4`.
- Exact-file Rustfmt and Python compilation pass for the production, lower, and
  contract files.
- The batched nine-contract Runtime/Editor loader passes `36/36` tests in
  `0.025s` with zero failures, errors, or skips. The refreshed eleven-contract
  loader passes `44/44` tests in `0.047s`; the broad non-tooling
  performance/pressure loader passes `2402/2402` tests across `656` modules in
  `33.492s`, with zero load errors, failures, errors, or skips.
  Managed Windows Cargo/Release, allocator, and Editor inspector product
  p50/p95/p99 evidence remain pending behind the external worktree admission
  gate.
  A subsequent single-process recheck passes the same eleven-contract slice
  `44/44` in `0.017s` and the broad batch `2402/2402` across `656` modules in
  `6.455s`; these remain local source/model receipts.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/ui/pane_data_conversion/ui_asset_detail_fields/widget.rs` | `37B5EC6552E34A041953F8F81ECA6FE331ECA0A7C2A3980C46BE4EBBCC7C5DE7` |
| `zircon_editor/src/ui/retained_host/ui/pane_data_conversion/ui_asset_detail_fields/widget/capacity_tests.rs` | `7C7ED9A79B1769C0742C58148CEB47187701A213327D93EADD118A8521FA8BCB` |
| `tools/tests/test_editor_widget_detail_row_capacity_performance_contract.py` | `2D3B86A27CF59794BCAE3ACA69FD9F33546A8A4E18928E997939213D30645B21` |

## Acceptance boundary

Keep this record `implementation_complete` /
`managed_validation_pending` until the owner-attributed managed Windows
Release lane compiles the current Runtime/Editor tree, executes the lower
regression and ignored marker, and supplies allocator plus Editor inspector
product p50/p95/p99 evidence. Tooling production remains deferred for the
later Rust migration.
