---
title: WOS239 Counter Shot ranged interrupt lockout runtime
status: implementation_complete_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS239 Counter Shot ranged interrupt lockout runtime

## Objective

Project the source-pinned Hunter `counter_shot` ability into WOC M4 and close
its offline combat loop: level-10 instant cast, 35-resource cost, 20-second
cooldown, normal GCD, 8-35 hostile target/facing admission, immediate
interruption of a valid active non-physical cast, and the interrupted school's
four-second lockout.

## Source contract

- Source: `dev/world-of-claudecraft/src/sim/content/classes.ts` at
  `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`.
- `counter_shot` is a physical Hunter ability with cost `35`, cast time `0`,
  cooldown `20`, range `35`, minimum range `8`, `scalesWith: "ranged"`, a
  target requirement, and one `interrupt` effect with `lockout: 4`.
- The source physical ability is not marked as a projectile, so its interrupt
  effect resolves at cast admission, not at projectile impact.
- `effect_dispatch.ts` only cancels a target's active, interruptible,
  non-physical cast; it applies a school-keyed lockout after cancellation when
  the existing crowd-control policy admits it. A valid target with no active
  cast still pays the ability cost and cooldown, but produces no lockout.

## Existing WOC support to reuse

- The Pummel reducer already models valid interrupt selection, cast
  cancellation, school lockout motion aura, expiration, and WOS114 serialization.
- WOS222 owns the separate scripted pseudo-cast identity bridge. The retained
  Eastbrook target gate admits only NPC targets, so its player-versus-player
  lockout DR and unavailable scripted channel identity do not alter this slice.
- Existing target/facing math provides ranged Hunter min/max admission.
- The M4 generator already projects the `interrupt` effect shape and supports
  source-pinned retained ability records.

## Delivery order

1. Retain Counter Shot in the generated M4 contract and update the fixed
   generator/coverage cardinalities.
2. Add a Hunter-specific profile, 8-35 target gate and instant reducer that
   spends cost and cooldown before delegating to the existing interrupt state.
3. Route slot and typed commands without adding a projectile queue branch.
4. Add a `zr_vm:project` fixture covering active magic cast cancellation and
   four-second school lockout, no-active-cast consumption, physical/fishing
   non-interruption, geometry rejection, snapshot and typed parity.
5. Run static generation/source guards and dispatch independent review; defer
   dynamic fixture acceptance to coordinator wakeup without polling.

## Non-goals

- No engine or ZrVM implementation changes.
- No projectile visual or delayed-impact behavior for Counter Shot.
- No changes to Pummel's established melee/rage behavior.

## Output record

| Slice | Scope | State | Date | Evidence |
| --- | --- | --- | --- | --- |
| WOS239 | Counter Shot ranged interrupt and school lockout | implementation_complete_static_validation_pending | 2026-08-03 | Source-pinned catalog and generated Zr projection contain Counter Shot at index 112. The Hunter-only 8-35 target gate, 35-resource instant admission, normal GCD and 20-second cooldown resolve before the shared interrupt predicate; a valid Frost cast is cancelled and receives a 4-second Frost lockout without an RNG draw or projectile. No-active, physical, fishing, geometry, serialization, slot and typed-command cases are covered by the `zr_vm:project` fixture. The forward P2 repair projects source `Bladestorm` as contract-only index 113 and uses that real `uninterruptible` target to prove reducer-level cost/GCD/cooldown consumption while retaining the target cast and producing no lockout. Source guards, all three generators, Node syntax and diff checks pass; the final independent review found no actionable findings. Dynamic acceptance remains coordinator-owned. |
