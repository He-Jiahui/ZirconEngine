---
title: WOS182 Cone of Cold runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS182 Cone of Cold Runtime

## Scope

Replicate the source Mage `cone_of_cold` Frost signature in the retained
offline world: instant self-centered spell AoE, standard GCD/cooldown/cost
transaction, spell-power and live spell-damage multiplier, deterministic
per-target rolls, combat/threat effects, and typed/slot command routing.

## Source Transaction

- Cone of Cold is Mage level 10, Frost school, zero cast time, 60 resource
  cost, 20 second cooldown, and a normal GCD. It is not spec-gated.
- The effect is a radius-8 Frost `aoeDamage` with base roll [28, 36]. It does
  not set `canCrit`; an empty target list draws no RNG, while each eligible
  target consumes one range draw in source traversal order.
- It is a spell AoE, so active `buff_spelldmg` applies after spell-power and
  before rounding. It does not share the DoT snapshot path or physical armor
  reduction.

## Delivery Order

1. Add the source-pinned ability identity/profile and slot/typed routes.
2. Reuse the existing instant-AoE reducer with live spell multiplier, then
   preserve existing break-control, combat, threat, cost, cooldown, and RNG
   semantics without broadening `groundAoE`.
3. Add focused `zr_vm:project` package and static guard, followed by a second
   review. Dynamic evidence remains exclusive to `zr_vm:project`.

## Status

| Milestone | Scope | Status | Date | Evidence |
|---|---|---|---|---|
| WOS182a | Cone of Cold source transaction | implemented_static_validation_pending | 2026-08-01 | `node examples/woc/tools/wos182_cone_of_cold_runtime_static_guard.mjs`; source pin, generated catalog, payload, state transaction, snapshot test |
| WOS182b | Independent secondary review | completed | 2026-08-01 | No findings: verified source values, post-spell-power live multiplier, inclusive radius, one range draw per eligible target, source-order target traversal, cost/GCD/cooldown atomicity, threat/combat transition, and whiff behavior |

## Dynamic Validation

The focused package is
`examples/woc/scripts/woc_game/woc_m4_cone_of_cold_runtime_tests.zrp`.
It must be executed only through `zr_vm:project`. The currently available
environment does not expose that backend, so this remains an explicit pending
dynamic validation item; no alternate runtime was used.
