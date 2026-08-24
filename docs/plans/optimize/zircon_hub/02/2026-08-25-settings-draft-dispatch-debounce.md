---
title: Hub 02 Settings Draft Dispatch Debounce
category: zircon_hub
report_id: Hub02-settings-draft-dispatch-debounce-2026-08-25
date: 2026-08-25
session_id: optimize-hub02-catalog-linear-grouping-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_passed
---

# Hub 02 Settings Draft Dispatch Debounce

## Scope

This batch addresses the Hub02 finding that every settings keystroke publishes the complete draft
through the Tauri action boundary. It retains local immediate input feedback and the existing full
draft payload contract. It does not claim that settings revision/CAS, field patches, persistence
transactions, or the parent plan's complete settings acceptance gates are finished.

## Change

`SettingsPage` now schedules draft publication after a 200 ms quiet window. A newer input replaces
the pending timer and draft, so only the latest complete draft crosses the action boundary. The
dispatcher keeps the current React callback through a ref and cancels pending work on unmount.

Save and folder-browse actions already carry the complete local draft, so they cancel the delayed
publication before dispatch. Discard and restore-default actions also cancel it. This prevents a
stale timer from publishing the pre-decision draft after one of those explicit workflow boundaries.

## Performance

The deterministic burst gate runs 21 samples of 100 successive complete-draft updates:

- legacy complete-draft action dispatches: `2,100`;
- optimized complete-draft action dispatches: `21`;
- dispatch reduction: `99.000%`;
- pending timers per burst after the last input: `1`;
- stale publications after cancellation: `0`.

The acceptance threshold is exactly one optimized dispatch per 100-input burst. This count gate is
preferred over wall-clock timing because the improvement removes cross-process serialization and
backend work whose latency is environment-dependent.

## Validation

- RED: the direct Node test failed because the debounce helper did not exist.
- GREEN: 3/3 direct Node tests pass with Node 22 type stripping enabled.
- The behavior gate covers last-draft publication, cancellation, and SettingsPage workflow-boundary
  wiring.
- Coordinator ticket `0825f9161c9e4d8f9a46eaae0075e0fa` restored pinned dependencies, passed
  TypeScript typecheck, and passed the combined catalog/settings batch.
- The managed settings row reports `2,100 -> 21` complete-draft dispatches (`99.000%` lower), one
  dispatch per 100-update burst, and zero stale publications after cancellation.
- The combined ticket passed all six direct Node tests and three performance rows. Final record seal,
  commit, and WeCom publication are pending.
