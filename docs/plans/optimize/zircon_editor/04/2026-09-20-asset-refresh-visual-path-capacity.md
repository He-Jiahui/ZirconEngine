---
title: Editor844 Asset Refresh Visual Path Capacity
category: zircon_editor
report_id: Editor844-asset-refresh-visual-path-capacity-2026-09-20
date: 2026-09-20
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor844 · asset refresh visual-path capacity

## Scope

The retained Editor asset-refresh path gathers visual locators from runtime,
editor-asset, and resource change streams before sorting and deduplicating them.
The previous collector started with zero capacity even though each stream has a
finite event count in the current refresh batch.

## Optimization

- Compute a saturating two-sided locator bound for runtime/resource changes and
  a one-sided bound for editor-asset changes.
- Keep the empty/non-visual path allocation-free, then reserve the bound only
  when the first actual visual locator is accepted.
- Preserve sprite-atlas full invalidation, lagged-resource reconciliation,
  locator order before sorting, sorting/deduplication, and `None` semantics.

## TDD and deterministic evidence

The Python source/model contract was intentionally run RED before the lower
module and production reservation existed, then GREEN after both were wired
(`4/4`). The lower Rust module covers lazy capacity, non-visual zero-capacity,
sort/dedup behavior, and the ignored
`EDITOR844_ASSET_REFRESH_VISUAL_PATH_CAPACITY_BENCH_V1` Release marker. A
representative `(8, 7, 6)` event batch changes the bounded collector model from
`6→0` geometric growth events.

## Local validation

- `tools/tests/test_editor_asset_refresh_visual_path_capacity_performance_contract.py`:
  `4/4`.
- Exact-file Rustfmt and Python compilation pass for the production, lower, and
  contract files.
- The combined focused Runtime/Editor loader passes `82/82` tests across `21`
  modules in `0.089s`; the broad non-tooling performance/pressure loader passes
  `2370/2370` tests across `648` modules in `4.907s`, with zero load errors,
  failures, errors, or skips. Managed Windows Cargo/Release, allocator, and
  asset-refresh product p50/p95/p99 evidence remain pending behind the external
  worktree admission gate.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/app/assets/refresh.rs` | `4084F51E10703D8BDF898F4EF62AA0005DCA9A67ED287F16BB7989181B7FA3FB` |
| `zircon_editor/src/ui/retained_host/app/assets/refresh/capacity_tests.rs` | `5783FC8D8287118250F28FF64579EC1D12D310CBCAD588172B5CFD0EE272957E` |
| `tools/tests/test_editor_asset_refresh_visual_path_capacity_performance_contract.py` | `7B72F9C641A5415B746678E35F689AB022A48CB2834F9C7886652881F52AED26` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed managed Windows Release lane compiles the current
Runtime/Editor tree, executes the lower regression and ignored marker, and
supplies allocator plus asset-refresh product p50/p95/p99 evidence. Tooling
production remains deferred for the later Rust migration.
