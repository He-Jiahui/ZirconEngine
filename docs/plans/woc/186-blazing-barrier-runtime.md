---
title: WOS186 Blazing Barrier runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS186 Blazing Barrier Runtime

## Scope

Replicate Fire Mage `blazing_barrier`: level-5 Fire self shield, 90 resource,
normal GCD, 30-second cooldown, and a 130-point 60-second Fire absorb aura.
It completes the Fire half of the source `personal_barrier` slot used by Warded.

## Delivery Order

1. Add a focused `zr_vm:project` test before implementation.
2. Reuse the bounded two-personal-barrier absorb contract, Fire school lockout
   routing, typed/slot commands, and Warded's source sentinel.
3. Add static guard and secondary review; no alternate runtime.

## Status

| Milestone | Scope | Status | Date |
|---|---|---|---|
| WOS186a | Test contract and plan | completed | 2026-08-01 |
| WOS186b | Runtime implementation | implemented_static_validation_pending | 2026-08-01 |
| WOS186c | Independent secondary review | completed | 2026-08-01 |

## Dynamic Validation

`examples/woc/scripts/woc_game/woc_m4_blazing_barrier_runtime_tests.zrp` runs
only with `zr_vm:project`.
