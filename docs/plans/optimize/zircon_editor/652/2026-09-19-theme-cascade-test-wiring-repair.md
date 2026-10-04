---
title: Editor652 theme cascade lower-test wiring repair
category: zircon_editor
report_id: Editor822-theme-cascade-test-wiring-repair-2026-09-19
date: 2026-09-19
related_to:
  - docs/plans/optimize/zircon_editor/652/2026-09-01-preallocated-theme-cascade-outputs.md
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: lower_regression_wired
---

# Editor822 · theme cascade lower-test wiring repair

## Finding

The Editor652 lower regression file existed at
`style/theme_cascade_inspection/optimization_batch_jm_editor652_tests.rs`, but
`theme_cascade_inspection.rs` did not declare the child test module. The capacity
contract and ignored Release marker were therefore detached from the Rust test tree.

## Repair

The production owner now declares the existing test-only module with its explicit
path. The cascade projection implementation is unchanged; layer, token, and rule
capacity behavior remains the Editor652 implementation.

## TDD and local evidence

- RED: the strengthened source contract failed because the path declaration and
  module declaration were absent.
- GREEN: the focused contract passes `3/3`; exact Rustfmt passes for the production
  owner and lower regression file.
- A four-slice Runtime/Editor core batch (Runtime819, Runtime820, Editor819, and
  Editor822) passes `12/12` in one process with zero failures, errors, or skips.
- The lower regression and ignored `EDITOR652_CASCADE_OUTPUT_CAPACITY_BENCH_V1`
  marker are now reachable by Rust test discovery.
- The current one-process Runtime/Editor performance-contract loader now covers
  `390` modules and passes `1410/1410` tests with zero failures, errors, or skips.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/asset_editor/style/theme_cascade_inspection.rs` | `DCB291DED7C995F2E9FB42B29453BE6307AB05F146E778ACC30CD6497986F929` |
| `zircon_editor/src/ui/asset_editor/style/theme_cascade_inspection/optimization_batch_jm_editor652_tests.rs` | `799F2CCB3FB6AF2A8B04E3B1747CE8D667D765F53493BB595528915E29699928` |
| `tools/tests/test_editor_theme_cascade_output_capacity_performance_contract.py` | `921F68908E51E1EDB413224526928C00187E6D8A465905A332D305D1B663AE68` |

## Acceptance boundary

Managed Windows Cargo/Release compilation, execution of the now-wired lower
regression and ignored marker, allocator observations, and Editor asset-editor
p50/p95/p99 evidence remain pending behind the external dirty-worktree gate.
Tooling production remains deferred for the later Rust migration.
