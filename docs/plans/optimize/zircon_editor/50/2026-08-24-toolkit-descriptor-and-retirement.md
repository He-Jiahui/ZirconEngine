---
title: Editor50 Toolkit Descriptor and Retirement Hardening
category: zircon_editor
report_id: Editor50-toolkit-descriptor-retirement-2026-08-24
date: 2026-08-24
session_id: root-editor50-toolkit-descriptor-r2-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor50 Toolkit Descriptor and Retirement Hardening

## Scope

This slice addresses the `DocumentToolkitRegistry` portion of Editor50 P0-05. It freezes toolkit
descriptors outside the registry mutex and retires callback-owned toolkit objects after unlocking.
It does not claim the parent plan's document/dirty-registry transaction, callback supervisor,
panic conversion, extension reconciler, or cross-registry mount lifecycle is complete.

## Implementation

Registration calls the external `descriptor()` trait method once before acquiring the registry
mutex. Each registry entry owns that descriptor, so snapshot publication, save reporting, clear,
and close operate on pure cached data without invoking toolkit code under the lock.

`clear` moves the retired entry map out of the state, publishes the empty snapshot, and releases the
mutex before dropping the map. `commit_close` follows the same order for its removed entry. Focused
regressions make toolkit `Drop` re-enter `snapshot()` through both paths and require completion
within two seconds, proving that object retirement no longer self-deadlocks on the registry mutex.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| Register and clear 1,000 toolkits | 501,500 external descriptor calls | 1,000 calls; <= 3 s | 99.8006% callback reduction |
| Snapshot publication callback work | O(toolkits) external calls per publication | 0 external calls | cached owned descriptors only |
| Clear/close callback-owned object destruction | while holding registry mutex | after mutex release; <= 2 s re-entry bound | removes self-deadlock path |

The ignored Windows-native release evidence prints `EDITOR_TOOLKIT_BENCH_V1` with exact elapsed
nanoseconds and the descriptor-call reduction. Runtime elapsed values are accepted only from
coordinator terminal evidence.

## Validation

- Exact `rustfmt --check`, scoped `git diff --check`, descriptor-call/source contracts, reentrant
  clear and unregister regressions, and ignored release evidence are prepared for a shared
  coordinator batch with another Runtime or Editor optimization.
- No local Cargo lane is launched and no compilation is monitored in real time.
- Final validation ticket, terminal marker values, and commit integration remain pending.

## Remaining Parent-plan Work

Toolkit and dirty-state registration are still separate commits, toolkit callbacks do not yet have
a typed panic/fault supervisor, `autosave_source_path` does not hold the document revision lease,
and the wider owner-generation mount/revoke transaction remains open in Editor50.
