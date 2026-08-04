---
title: WOS238 Concussive Shot physical projectile slow runtime
status: implementation_complete_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS238 Concussive Shot physical projectile slow runtime

## Objective

Project the source-pinned Hunter `concussive_shot` ability into the WOC M4
catalog and implement its complete offline combat loop: level-8 cast gating,
20-resource instant cast, 12-second cooldown, 8-35 range validation,
ranged-attack-power physical projectile damage, and a 0.5 movement multiplier
for four seconds after a surviving target is struck.

## Source contract

- Source: `dev/world-of-claudecraft/src/sim/content/classes.ts` at
  `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`.
- `concussive_shot` is a Hunter level-8, target-required, instant physical
  projectile with cost `20`, cooldown `12`, range `35`, minimum range `8`, and
  `scalesWith: "ranged"`.
- The ordered effects are a `directDamage` range of `4..6`, followed by a
  `slow` with multiplier `0.5` and duration `4` seconds.
- `casting_lifecycle.ts` keeps normal ranged range validation for a positive
  range ability. `effect_dispatch.ts` resolves physical direct damage with
  ranged attack power, physical criticals, armor mitigation, and authoritative
  range/critical draws; it dispatches slow only after the target survives the
  direct-damage effect.

## Existing WOC support to reuse

- The generated M4 catalog and projection coverage scripts own the catalog
  projection contract.
- Aimed Shot provides the existing delayed physical ranged projectile lifecycle
  and source-target snapshot conventions.
- `combat/spell_scaling.zr`, `combat/mob_swing_state.zr`, and the established
  critical helpers provide WOC physical damage primitives.
- The serializable offline motion-aura state already supports physical slows,
  source snapshots, expiration, and WOS114 codec coverage.

## Delivery order

1. Add the source-pinned catalog/projection record and a focused static guard.
2. Isolate Concussive Shot's pure physical projectile direct-damage calculation
   in a combat module; keep world-state mutation, cast gates, queueing, and
   aura application in `world/state.zr`.
3. Add slot and typed-cast routing, preserving cast-time source snapshots
   through projectile landing.
4. Add a `zr_vm:project` runtime fixture covering cost, range/min-range,
   cooldown, two authoritative draws, armor, target death, threat, and the
   surviving-target slow/aura snapshot.
5. Run only static code-generation, source-contract, syntax, and diff checks;
   dispatch independent review, then leave dynamic acceptance to coordinator
   wakeup without polling.

## Non-goals

- No engine, renderer, or ZrVM implementation changes.
- No fallback runtime backend.
- No unrelated Aimed Shot refactor; shared pure logic is introduced only when
  it reduces the Concussive Shot state-surface complexity without changing its
  established behavior.

## Output record

| Slice | Scope | State | Date | Evidence |
| --- | --- | --- | --- | --- |
| WOS238 | Concussive Shot physical ranged projectile and slow | implementation_complete_static_validation_pending | 2026-08-03 | Source-pinned catalog and generated Zr projection are consistent at 113 abilities; instant cast spends 20 resource, arms 12-second cooldown and queues a physical projectile; landing owns the exact ranged-AP coefficient, physical 2x critical, armor, threat and surviving-target 0.5/4-second slow. Slot/typed routes, serialization snapshot, fizzle, lethal and geometry cases are covered by a `zr_vm:project` fixture. Static generators, source guard, Node syntax and diff checks pass. Two forward P2 test-coverage repairs received clean independent re-review; dynamic fixture acceptance remains coordinator-owned. |
