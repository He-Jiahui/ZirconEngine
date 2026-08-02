---
title: WOS184 Ice Barrier runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS184 Ice Barrier Runtime

## Scope

Replicate source Mage Frost `ice_barrier` (Frostveil) in the retained offline
world: level-5 Frost admission, instant self-cast, normal resource/GCD/cooldown
transaction, and the source `absorb` aura with a 130-point, 60-second profile.

## Source Transaction

- The source ability costs 90, has zero cast time, a 30-second cooldown, Frost
  school, no target, and is Frost-spec gated.
- Its normal GCD follows the shared source floor of 0.75 seconds after spell
  haste; it is not subject to the obsolete 1.0-second Mage-only cap.
- Its sole effect applies an absorb aura to the caster. The aura identity is
  `(target, source, ability)`, so a fresh same-source cast replaces the old row
  before it is appended; normal incoming damage consumes recent auras first.
- Ice Barrier is outside the M4 generated ability catalog. Its source-pinned
  fixed profile must be admitted beside, not folded into, the generated PWS
  profile path.

## Delivery Order

1. Add the focused `zr_vm:project` test package before implementation.
2. Add source-pinned identity/profile, Frost school routing, typed and slot
   routes, then generalize the compact absorb-row reducer only to PWS and Ice
   Barrier.
3. Add a static guard and complete an independent second review. Dynamic
   validation remains exclusive to `zr_vm:project`.

## Status

| Milestone | Scope | Status | Date | Evidence |
|---|---|---|---|---|
| WOS184a | Test contract and plan | completed | 2026-08-01 | focused package captures spec gate, resource/GCD/cooldown, self absorb identity, damage consumption, refresh, routes, and snapshot |
| WOS184b | Runtime implementation | implemented_static_validation_pending | 2026-08-01 | `node examples/woc/tools/wos184_ice_barrier_runtime_static_guard.mjs`; Frost gate, resource/GCD/cooldown, self identity, reverse absorb, source refresh, routes, snapshot |
| WOS184c | Independent secondary review | completed | 2026-08-01 | Found the shared Mage GCD floor incorrectly pinned at 1.0s; forward-fixed to source `MIN_GCD` 0.75s and re-ran WOS86 plus WOS179-WOS184 guards. No remaining baseline transaction discrepancy. |

## Follow-on Boundary

The source talent layers `barrierDrPct` (Warded) before personal-barrier
absorption and `manaDefCdrPer10` (Overflowing Power) after mana spend. They
are cross-ability talent behavior rather than part of this baseline ability
transaction, and continue in WOS185 without treating them as engine
infrastructure gaps.

## Dynamic Validation

The focused package is
`examples/woc/scripts/woc_game/woc_m4_ice_barrier_runtime_tests.zrp`. It must
be executed only through `zr_vm:project`; this environment does not expose that
backend, and no alternate runtime will be used.
