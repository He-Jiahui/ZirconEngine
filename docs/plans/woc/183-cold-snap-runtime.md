---
title: WOS183 Cold Snap runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS183 Cold Snap Runtime

## Scope

Replicate source Mage `cold_snap` (Winter's Recall) in the retained offline
world: its level-17, zero-cost, instant, off-GCD, 120-second transaction and
the exact five-entry cooldown reset list.

## Source Transaction

- The source ability is Frost school, targetless, off the global cooldown, and
  clears only `blink`, `ice_barrier`, `blazing_barrier`, `temporal_barrier`,
  and `greater_invisibility`.
- Source `clearCooldowns` deletes every listed cooldown even if the ability is
  not currently known. It neither resets Cold Snap itself nor changes the
  global-cooldown state. `offGcd` bypasses the GCD only; the normal busy-cast
  guard still rejects Cold Snap because it has no `usableWhileCasting` exception.

## Delivery Order

1. Add the focused `zr_vm:project` test package before implementation.
2. Add source-pinned identity/profile, Frost school routing, typed and slot
   command routes, then reuse the existing compacting cooldown deletion helper.
3. Add a static guard and perform an independent second review. Dynamic
   validation remains exclusive to `zr_vm:project`.

## Status

| Milestone | Scope | Status | Date | Evidence |
|---|---|---|---|---|
| WOS183a | Test contract and plan | completed | 2026-08-01 | focused package records level gate, exact reset set, off-GCD, routing, cooldown gate, and snapshot requirements |
| WOS183b | Runtime implementation | implemented_static_validation_pending | 2026-08-01 | `node examples/woc/tools/wos183_cold_snap_runtime_static_guard.mjs`; level/busy/GCD gates, exact reset list, source order, typed and slot routing, snapshot |
| WOS183c | Independent secondary review | completed | 2026-08-01 | One busy-cast omission found against `casting_lifecycle.ts`, forward-fixed; post-fix review found no remaining discrepancies in cost, cooldown, GCD, reset set, lockout school, or snapshot behavior |

## Dynamic Validation

The focused package is
`examples/woc/scripts/woc_game/woc_m4_cold_snap_runtime_tests.zrp`.
It must be executed only through `zr_vm:project`. The current environment does
not expose that backend, so it remains an explicit pending dynamic validation
item; no alternate runtime was used.
