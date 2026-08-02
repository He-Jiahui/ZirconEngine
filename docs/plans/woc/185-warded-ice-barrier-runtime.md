---
title: WOS185 Warded Ice Barrier runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS185 Warded Ice Barrier Runtime

## Scope

Replicate the level-8 Mage `mag_r8_warded` selection while Frostveil is active:
15% pre-absorb damage reduction and its `mag_warded` 39-point shield-consumed
heal through the existing critical-heal and effective-healing-threat bridge.

## Source Transaction

- Warded sets `barrierDrPct: 0.15`; source applies the rounded reduction before
  any absorb aura consumes the incoming damage.
- `mag_warded` listens for `personal_barrier` consumption. It matches Frostveil
  and heals the barrier subject for 39 after the absorb row is fully depleted.
- The heal is a normal source heal: one crit roll, effective-healing clamp and
  split 0.5-per-effective-point threat across hostile combatants aware of the
  healed target.

## Delivery Order

1. Add focused `zr_vm:project` test and source-pinned plan before implementation.
2. Reuse current talent modifier/proc catalogs, Ice Barrier absorb identity and
   the existing `healState` kernel without adding parallel talent state.
3. Add a static guard and second review; retain `zr_vm:project` as the sole
   dynamic backend.

## Status

| Milestone | Scope | Status | Date | Evidence |
|---|---|---|---|---|
| WOS185a | Test contract and plan | completed | 2026-08-01 | test captures pre-absorb reduction, full-consume heal/RNG/threat, unselected control, and snapshot |
| WOS185b | Runtime implementation | implemented_static_validation_pending | 2026-08-01 | `node examples/woc/tools/wos185_warded_ice_barrier_runtime_static_guard.mjs`; generated selection/proc lookup, pre-absorb reduction, consumed-shield heal, crit RNG, effective healing threat, snapshot |
| WOS185c | Independent secondary review | completed | 2026-08-01 | Verified `Math.round` equivalence for nonnegative damage, source ordering before absorb, exact one-RNG normal-heal semantics only on depletion, and source-aware effective-healing threat. No scoped discrepancies found. |

## Dynamic Validation

`examples/woc/scripts/woc_game/woc_m4_warded_ice_barrier_runtime_tests.zrp`
must run only with `zr_vm:project`. No alternative runtime will be used.
