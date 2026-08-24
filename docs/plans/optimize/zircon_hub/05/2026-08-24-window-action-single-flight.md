---
title: Hub 05 Window Action Single-flight Performance
category: zircon_hub
report_id: Hub05-window-action-single-flight-2026-08-24
date: 2026-08-24
session_id: optimize-hub05-window-action-single-flight-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_passed
---

# Hub 05 Window Action Single-flight Performance

## Scope

This batch advances the Hub 05 window IPC and failure-observability work. It covers duplicate native
window commands and visible failure projection for minimize, toggle-maximize, and close. It does not
claim the parent plan's full bootstrap, shutdown, crash-recovery, watchdog, or process-exit contract
is complete.

## Change

The frameless window controls previously dispatched every click directly to Tauri and discarded the
returned promise. `createWindowActionScheduler` now keeps one in-flight receipt per action kind.
Repeated commands of the same kind share that receipt, while different action kinds remain
independent. Settlement releases the slot for retries. Synchronous and asynchronous failures are
reported once through the App failure handler and projected into the existing task error surface.

The scheduler is retained in a React ref, so unrelated renders do not reset single-flight state.
The failure callback is separately updated through a ref, avoiding stale shell text or state while
preserving scheduler identity.

## Performance Evidence

The deterministic burst gate runs 21 groups of 100 simultaneous close requests:

- legacy native dispatches: `2,100`;
- optimized native dispatches: `21`;
- duplicate native dispatch reduction: `99.000%`;
- maximum optimized dispatches per burst: `1`.

This is deterministic work-count evidence rather than a wall-clock timing claim; native window IPC
latency depends on the desktop compositor and OS state.

## Validation

- Local Node behavior/performance batch: 3/3 passed.
- Coalescing preserves independent action kinds and returns the same receipt to duplicates.
- Sync/async failures are reported once; retries are admitted after settlement.
- Scoped whitespace validation: passed.
- Managed dependency restore, production build/typecheck, Rust source contracts, and the 3/3 Node
  batch passed in coordinator ticket `1ff81bc347124334a81fcb0483e5d313`.
- No Cargo lane or Cargo dry-run was launched, polled, or terminated.

## Remaining Parent-plan Work

Hub 05 still requires the authoritative application host lifecycle, bounded startup and shutdown
deadlines, exit disposition, crash/restart recovery, window state persistence, and product-level
fault-injection qualification.
