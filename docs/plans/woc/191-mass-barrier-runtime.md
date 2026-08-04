---
title: WOS191 Mass Barrier runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS191 Mass Barrier Runtime

## Scope

Replicate source Mage `mass_barrier`: the level-17 talent grants an instant,
90-second, 150-resource Frost spell which shields the caster and up to four
living party/raid players within 30 yards. Each selected recipient receives a
130-point absorb for 60 seconds. Candidate enumeration is deterministic by
entity id; only when more than five candidates exist does source ordering put
the caster first, then distance squared, then entity id before truncation. A
solo caster still receives their shield.

The existing WOS party rows and bounded absorb rows own this behavior; no
engine service or new runtime capability is required. The source's personal
barrier cooldown coupling is retained: casting Mass Barrier advances the
matching Arcane personal barrier to at least 12 seconds, or the Fire/Frost
personal barrier to at least 30 seconds. Its aura school follows the selected Mage specialization in the
source presentation layer; the retained offline absorb model has no aura
school column, so this slice preserves the authoritative targeting, duration,
amount, resource and cooldown state rather than inventing a presentation-only
state shape.

WOS190 applies as well: an armed Overload amplifies each positive absorb to
182 and bills 225 resource at cast resolution.

## Source Contract

- `classes.ts` declares `aoeAllyAbsorb(130, 60, 30, maxTargets: 5)` with
  Frost school, zero cast time and a 90-second cooldown.
- `group_targeting.ts` includes only living player party/raid members in
  radius (or the caster alone) and has no RNG.
- `effect_dispatch.ts` limits over-cap recipients by caster, distance squared
  and entity id, then applies the identity `(target, source, mass_barrier)`.
- The source chooses `temporal_barrier`, `blazing_barrier` or `ice_barrier`
  from Arcane, Fire or Frost specialization and takes the larger existing
  personal cooldown (12 seconds for Arcane; 30 seconds for Fire/Frost).

## Delivery Order

1. Add focused `zr_vm:project` entry coverage for talent admission, recipient
   ordering/cap, out-of-range/dead rejection, resource/GCD/cooldown, personal
   barrier coupling, Overload and snapshot restore.
2. Add source-pinned profile helpers, party recipient selection and the
   instant cast reducer using the existing absorb rows.
3. Route slot and typed casts, extend WOS190 coverage and add static guards.
4. Run static regression and a second independent review. Dynamic validation
   remains exclusively `zr_vm:project`.

## Dynamic Validation

`examples/woc/scripts/woc_game/woc_m4_mass_barrier_runtime_tests.zrp` must
run only through `zr_vm:project`. No alternate runtime is permitted.

## Status

| Milestone | Scope | Status | Date |
|---|---|---|---|
| WOS191a | Source contract, red static guard and dynamic entry | completed | 2026-08-02 |
| WOS191b | Runtime reducer, party/raid selection, routing and snapshot coverage | implemented_static_validation_pending | 2026-08-02 |
| WOS191c | Static regression and second review | completed | 2026-08-02 |
