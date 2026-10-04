---
related_code:
  - zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/chrome_projection.rs
  - zircon_editor/src/ui/workbench/model/status_bar_model.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/chrome_projection.rs
  - zircon_editor/src/ui/workbench/model/status_bar_model.rs
plan_sources:
  - user: 2026-09-21 optimize Runtime and Editor hot paths and record completion
tests:
  - zircon_editor/src/ui/workbench/model/status_bar_viewport_chrome_text_tests.rs
  - tools/tests/test_editor862_viewport_chrome_direct_settings_performance_contract.py
doc_type: milestone-detail
title: Editor862 viewport chrome direct settings projection
category: zircon_editor
report_id: Editor862-viewport-chrome-direct-settings-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor862 - viewport chrome direct settings projection

## Scope

Each viewport-chrome sync needs only the current scene viewport settings plus
the grid and snap labels derived from them. The old path built the complete
Chrome snapshot and full `StatusBarModel`, including unrelated inspector,
project, asset, console, layout, and task state.

## Optimization

- Read the small authoritative `SceneViewportChromeSettings` projection already
  exposed by the event controller.
- Build viewport presentation data and the two required labels directly from
  that settings value, eliminating full Chrome and status-bar construction.
- Centralize grid/snap label generation in
  `StatusBarModel::viewport_chrome_text` so full status projection and the
  focused viewport patch retain one implementation.
- Preserve viewport conversion ownership, damage reporting, and native-window
  presenter patch behavior.

## TDD and deterministic performance evidence

The source/model contract was run RED before the focused settings path existed
and then GREEN at `4/4`. Lower regressions preserve Hidden, VisibleNoSnap, and
VisibleAndSnap label semantics, including formatted translate steps. The ignored
`EDITOR862_VIEWPORT_CHROME_DIRECT_SETTINGS_BENCH_V1` marker exercises 65,536
focused projections. The structural target changes full Chrome snapshot and
full status-model builds per viewport-chrome sync from `1` to `0`.

## Local validation

- The combined Editor859-862, related Workbench projection, and Runtime08d/861/862
  source-contract batch passes `43/43` in `0.036s` with zero failures, errors, or
  skips.
- The one-process current-tree explicit performance-or-contract loader covers
  `974` non-tooling files and passes `4121/4121` tests in `221.095s`, with zero
  load errors, failures, errors, or skips.
- After a transient Windows mapped-file write lock, exact-file read-only
  Rustfmt checks pass for the status-model owner, lower regression, and retained
  projection caller.
- No local Cargo command or coordinator poll was started. Managed Windows
  Release compilation, lower-test execution, allocator evidence, and viewport
  chrome p50/p95/p99 measurements remain pending.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/workbench/model/status_bar_model.rs` | `042B7B6FF815AD2AE8F171FDC17A69A3B934633D7FB36C4DDD59A538FE263E8D` |
| `zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/chrome_projection.rs` | `4A815B9CD4758F0B9BEF1E744BB12AFA17D67D482DEB559F6AEB7E41A77C78AE` |
| `zircon_editor/src/ui/workbench/model/status_bar_viewport_chrome_text_tests.rs` | `C0F9F05BBDDAAAE1D4A82B92AD487B0CCCB0E039543EEF283636D2C38606C1FF` |
| `tools/tests/test_editor862_viewport_chrome_direct_settings_performance_contract.py` | `022BD19F458C5BBE42D89B9267DFE73949B82A3B8ADD59D203BBFEB57A17B466` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release lane compiles the current Editor tree,
runs the lower regression and ignored marker, and supplies allocator plus
viewport-chrome p50/p95/p99 evidence. Tooling production stays deferred for the
later Rust migration.
